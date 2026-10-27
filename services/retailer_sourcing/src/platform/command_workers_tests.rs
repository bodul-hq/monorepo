//! Worker lifetime and PostgreSQL queue behavior.
use super::CommandWorkers;

#[tokio::test]
async fn joins_every_worker_even_if_an_earlier_worker_panics() {
    let (release, wait) = tokio::sync::oneshot::channel();
    let workers = CommandWorkers {
        handles: vec![
            tokio::spawn(async { panic!("fixture worker failure") }),
            tokio::spawn(async { wait.await.unwrap() }),
        ],
    };
    let joined = tokio::spawn(workers.join());
    tokio::task::yield_now().await;
    assert!(!joined.is_finished());
    release.send(()).unwrap();
    assert!(joined.await.unwrap().is_err());
}

mod postgres_tests {
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::Duration;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::platform::bus::{Bus, BusHandle, Command, Event, Handle};
    use crate::platform::state::AppState;
    use crate::platform::test_db::{Database, json_query};

    #[derive(Serialize, Deserialize)]
    struct Work {
        value: usize,
    }
    impl Command for Work {
        const NAME: &'static str = "WorkerTestWork";
    }
    #[derive(Serialize, Deserialize)]
    struct Worked {
        value: usize,
    }
    impl Event for Worked {
        const NAME: &'static str = "WorkerTestWorked";
    }

    #[derive(Default)]
    struct Gate {
        open: Mutex<bool>,
        released: Condvar,
    }
    impl Gate {
        fn release(&self) {
            *self.open.lock().unwrap() = true;
            self.released.notify_all();
        }
        fn wait(&self) {
            let (_guard, timeout) = self
                .released
                .wait_timeout_while(self.open.lock().unwrap(), Duration::from_secs(15), |open| {
                    !*open
                })
                .unwrap();
            assert!(!timeout.timed_out(), "test did not release the worker");
        }
    }
    struct ReleaseOnDrop(Arc<Gate>);
    impl Drop for ReleaseOnDrop {
        fn drop(&mut self) {
            self.0.release();
        }
    }
    struct Handler {
        gate: Arc<Gate>,
        started: tokio::sync::mpsc::UnboundedSender<usize>,
    }
    impl Handle for Handler {
        type Command = Work;
        type Event = Worked;
        type Error = &'static str;
        fn handle(&self, command: Work) -> Result<Vec<Worked>, Self::Error> {
            self.started.send(command.value).unwrap();
            self.gate.wait();
            Ok(vec![Worked {
                value: command.value,
            }])
        }
    }

    fn setup(
        db: &Database,
    ) -> (
        BusHandle,
        ReleaseOnDrop,
        tokio::sync::mpsc::UnboundedReceiver<usize>,
    ) {
        let (started, receiver) = tokio::sync::mpsc::unbounded_channel();
        let gate = Arc::new(Gate::default());
        let mut bus = Bus::default();
        bus.handle(Handler {
            gate: gate.clone(),
            started,
        });
        // An automatically consumed event would add another command.
        bus.on::<Worked, Work>(|event| {
            Some(Work {
                value: event.value + 1000,
            })
        });
        (
            bus.start(db.pool.clone()).unwrap(),
            ReleaseOnDrop(gate),
            receiver,
        )
    }
    fn rows(db: &Database, table: &str) -> Vec<Value> {
        json_query(
            &mut db.pool.get().unwrap(),
            &format!(
                "SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id), '[]') AS value FROM {table} t"
            ),
        )
        .as_array()
        .unwrap()
        .clone()
    }
    async fn received(receiver: &mut tokio::sync::mpsc::UnboundedReceiver<usize>) -> usize {
        tokio::time::timeout(Duration::from_secs(10), receiver.recv())
            .await
            .unwrap()
            .unwrap()
    }
    async fn completed(db: &Database, count: usize) {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                if rows(db, "command_entries")
                    .iter()
                    .filter(|row| row["status"] == 5)
                    .count()
                    == count
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    async fn zero_workers_leave_commands_for_the_manual_api() {
        let db = Database::new();
        let (bus, gate, mut started) = setup(&db);
        bus.dispatcher()
            .dispatch(&Work { value: 1 }, "test")
            .unwrap();
        let workers = bus.start_command_workers(0);
        // Wait longer than a poll interval: neither Bus::start nor zero workers consume.
        tokio::time::sleep(Duration::from_millis(1200)).await;
        assert!(started.try_recv().is_err());
        assert_eq!(rows(&db, "command_entries")[0]["status"], 0);
        gate.0.release();
        let response = crate::app(AppState {
            bus: Some(bus.clone()),
            ..Default::default()
        })
        .oneshot(
            Request::post("/api/commands/run")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"limit":1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(received(&mut started).await, 1);
        assert_eq!(rows(&db, "command_entries")[0]["status"], 5);
        assert_eq!(rows(&db, "event_entries")[0]["status"], 0);
        bus.shutdown();
        workers.join().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    async fn two_workers_reserve_distinct_batches_and_leave_events_queued() {
        let db = Database::new();
        let (bus, gate, mut started) = setup(&db);
        for value in 0..20 {
            bus.dispatcher().dispatch(&Work { value }, "test").unwrap();
        }
        let workers = bus.start_command_workers(2);
        let first = received(&mut started).await;
        let second = received(&mut started).await;
        assert_ne!(first, second);
        assert!(started.try_recv().is_err()); // Each worker's first handler is gated.
        gate.0.release();
        let mut values = vec![first, second];
        for _ in 2..20 {
            values.push(received(&mut started).await);
        }
        completed(&db, 20).await;
        bus.shutdown();
        workers.join().await.unwrap();
        values.sort_unstable();
        assert_eq!(values, (0..20).collect::<Vec<_>>());
        assert_eq!(rows(&db, "command_entries").len(), 20);
        let events = rows(&db, "event_entries");
        assert_eq!(events.len(), 20);
        assert!(events.iter().all(|row| row["status"] == 0));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "requires DATABASE_URL_TEST; run make test-postgres"]
    async fn shutdown_finishes_reserved_work_without_reserving_another_batch() {
        let db = Database::new();
        let (bus, gate, mut started) = setup(&db);
        bus.dispatcher()
            .dispatch(&Work { value: 1 }, "test")
            .unwrap();
        let workers = bus.start_command_workers(1);
        assert_eq!(received(&mut started).await, 1);
        // This command cannot be in the already-reserved batch.
        bus.dispatcher()
            .dispatch(&Work { value: 2 }, "test")
            .unwrap();
        bus.shutdown();
        let joined = tokio::spawn(workers.join());
        tokio::task::yield_now().await;
        assert!(!joined.is_finished());
        gate.0.release();
        tokio::time::timeout(Duration::from_secs(10), joined)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(started.try_recv().is_err());
        let commands = rows(&db, "command_entries");
        assert_eq!(commands.iter().filter(|row| row["status"] == 5).count(), 1);
        assert_eq!(commands.iter().filter(|row| row["status"] == 0).count(), 1);
        assert_eq!(rows(&db, "event_entries")[0]["status"], 0);
    }
}
