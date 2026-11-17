(globalThis.TURBOPACK||(globalThis.TURBOPACK=[])).push(["object"==typeof document?document.currentScript:void 0,174898,385065,e=>{"use strict";var t,r=e.i(226734),n=((t=n||{}).WSCH="wsch",t.NORMAL="normal",t),i="2025-07",a=class e{options;constructor(e){this.options={...e,apiVersion:e.apiVersion||i}}static fromEnv(){if(void 0===r.default||!r.default.env)throw Error("Environment variables not available");return new e({brand:"anker",environment:"production",storeTokens:JSON.parse('{"us":"70350afbd7faebe997a33340f0bc4843","ca":"0a32b45f34ceb286e6757f7b8e71b4b5","ca-fr":"0a32b45f34ceb286e6757f7b8e71b4b5","br":"081037f0ed2326b38c22401bd1b381eb","de":"05547607091975b6046fc6462349218b","se":"15b54cb3c0f67783f2b25bda3416006c","mx":"36c05d9daa3e3be0dd060d3602561e30","uk":"d224d651421261d351cb1bfee3a5fd5b","sea":"19953630a96be9c9660ce356fbbf62ad","es":"f419a33323122e2cb925e5927a421966","au":"ca89549ed256ec3f2c3a12d113b9f29e","nl":"e7ae5cd6d022c20de1720ee27c0de800"}'),storeDomains:JSON.parse('{"us":"ankerus.myshopify.com","ca":"anker-canada.myshopify.com","ca-fr":"anker-canada.myshopify.com","br":"anker-br.myshopify.com","de":"oneanker-de.myshopify.com","se":"oneanker-se.myshopify.com","mx":"oneanker-mx.myshopify.com","uk":"oneanker-uk.myshopify.com","sea":"oneanker-sea.myshopify.com","es":"oneanker-es.myshopify.com","au":"oneanker-au.myshopify.com","nl":"oneanker-nl.myshopify.com"}'),metafieldNamespacePrefix:"app--5059775--"})}getBrand(){return this.options.brand}getEnvironment(){return this.options.environment}getOriginUrl(){if(this.options.originUrl)return this.options.originUrl;let{brand:e,environment:t}=this.options;return"production"===t?`https://www.${e}.com`:`https://beta.${e}.com`}getPassportUrl(){if(this.options.passportUrl)return this.options.passportUrl;let{brand:e,environment:t}=this.options;return"production"===t?`https://passport.${e}.com`:`https://beta.passport.${e}.com`}getStorefrontToken(e){let t=this.options.storeTokens[e];if(!t)throw Error(`No storefront token found for locale: ${e}`);return t}getStoreDomain(e){let t=this.options.storeDomains[e];if(!t)throw Error(`No store domain found for locale: ${e}`);return t}getThemeDomain(e){let t=this.getStoreDomain(e),{environment:r,brand:n}=this.options;if("production"===r){let t={"eu-de":"de","eu-en":"eu",au:"au"}[e]||e;return`${t}.${n}.com`}return t}getApiUrl(e){let t=this.getStoreDomain(e),r=this.options.apiVersion||i;return`https://${t}/api/${r}/graphql.json`}getLocales(){return Object.keys(this.options.storeDomains)}hasLocale(e){return e in this.options.storeDomains}getCookieConfig(){return{domain:this.options.cookieOptions?.domain,expire:this.options.cookieOptions?.expire||30}}getApiVersion(){return this.options.apiVersion||i}getMetafieldNamespacePrefix(){return this.options.metafieldNamespacePrefix||("production"===this.getEnvironment()?"app--5059775--":"app--5059485--")}getCountryCode(e){return this.options.localeCountryMap?.[e]}getLanguageCode(e){return this.options.localeLanguageMap?.[e]}hasContextConfig(e){return!!(this.options.localeCountryMap?.[e]&&this.options.localeLanguageMap?.[e])}getOptions(){return{...this.options}}};e.s(["SOLD_OUT_PRICE",0,9999999.99,"ShopifyConfig",0,a,"atobID",0,function(e){return e&&"string"==typeof e&&e.includes("/")?e.split("/").pop()?.split("?")?.shift()||"":e},"btoaID",0,function(e,t="ProductVariant"){return`gid://shopify/${t}/${e}`},"deepClone",0,function e(t){if(null===t||"object"!=typeof t)return t;if(t instanceof Date)return new Date(t.getTime());if(t instanceof Array)return t.map(t=>e(t));let r={};for(let n in t)t.hasOwnProperty(n)&&(r[n]=e(t[n]));return r},"getLocalStorage",0,function(e){try{if("u"<typeof localStorage)return null;let t=localStorage.getItem(e);if(!t)return null;try{let r=JSON.parse(t);if(null!==r&&"object"==typeof r&&"value"in r&&"expires"in r){if(r.expires&&new Date().getTime()>r.expires)return localStorage.removeItem(e),null;return r.value}return r}catch{return t}}catch(t){return console.warn(`[getLocalStorage] Failed to get key "${e}":`,t),null}},"setLocalStorage",0,function(e,t,r){let n={value:t,expires:r?.expires?new Date(r.expires).getTime():null};try{"u">typeof localStorage&&localStorage.setItem(e,JSON.stringify(n))}catch(t){console.warn(`[setLocalStorage] Failed to set key "${e}":`,t)}}],385065);var o=["__discountCodeApp","discounts_function"];function u(e,t){return e&&0!==Object.keys(e).length?Object.entries(e).reduce((e,[r,n])=>{let i=`${t}combo`,a=n.map(e=>o.includes(e.namespace)||t&&e.namespace?.startsWith(t)?{namespace:e.namespace,key:e.key}:{namespace:i,key:e.namespace}).filter((e,t,r)=>t===r.findIndex(t=>t.namespace===e.namespace&&t.key===e.key));return e[`${r}MetafieldIdentifiers`]=a,e},{}):{}}var s=(e,t,r)=>{switch(e?.type&&e?.type.toLowerCase()){case"json":case"json_string":case"rating":case"volume":case"weight":case"dimension":if("string"==typeof e.value||"number"==typeof e.value)return JSON.parse(e.value);return e?.value||e;default:return e?.value||e}},l=e=>e?.reduce((e,t)=>{if(t)if(o.includes(t.namespace)){let r=t.namespace;e[r]=e[r]||{},e[r][t.key]=s(t)}else{let r=t.key;e[r]=e[r]||{};let n=s(t);n&&Object.entries(n).forEach(([t,n])=>{e[r][t]=e[r][t]??(n?s(n):null)})}return e},{}),d=class e{config;locale;constructor(e,t){this.config=e,this.locale=t}async request(e,t={}){let{query:r,variables:n,operationName:i}=e,{tags:a=[],...o}=t;r=function(e,t,r){if(!t||!r)return e;let n=e.match(/((?:query|mutation)\s+\w+)(\s*\([^)]*\))?\s*(\{)/);if(!n){let n=e.indexOf("{");if(-1===n)return e;let i=e.substring(0,n).trim(),a=e.substring(n);return`${i} @inContext(country: ${t}, language: ${r}) ${a}`}let i=n[1]+(n[2]||""),a=e.substring(n.index+n[0].length-1);return`${i} @inContext(country: ${t}, language: ${r}) ${a}`}(r,this.config.getCountryCode(this.locale),this.config.getLanguageCode(this.locale));let u=this.config.getApiUrl(this.locale),s={"Content-Type":"application/json","X-Shopify-Storefront-Access-Token":this.config.getStorefrontToken(this.locale),...o.headers||{}},l=JSON.stringify({query:r,variables:n,operationName:i});try{let e=await fetch(u,{method:"POST",headers:s,body:l,...o,...a.length>0&&{next:{tags:a}}});if(!e.ok)throw Error(`HTTP ${e.status}: ${e.statusText}`);let t=await e.json();return t.errors&&t.errors.length>0&&console.error("GraphQL Errors:",t.errors),t}catch(e){throw console.error("Shopify API Request Failed:",e),e}}async query(e,t,r){return(await this.request({query:e,variables:t},r)).data}getLocale(){return this.locale}getConfig(){return this.config}getApiUrl(){return this.config.getApiUrl(this.locale)}withLocale(t){return new e(this.config,t)}},c=`
  fragment image on Image {
    url
    altText
    width
    height
  }
`,f=`
  fragment seo on SEO {
    description
    title
  }
`,m=`
  fragment metafield on Metafield {
    value
    type
    description
    namespace
    key
  }
`,p=`
  fragment variant on ProductVariant {
    id
    sku
    title
    requiresShipping
    quantityAvailable
    currentlyNotInStock
    availableForSale
    barcode
    weight
    selectedOptions {
      name
      value
    }
    image {
      ...image
    }
    price {
      amount
      currencyCode
    }
    compareAtPrice {
      amount
      currencyCode
    }
    metafields(identifiers: $variantMetafieldIdentifiers) {
      ...metafield
    }
    sellingPlanAllocations(first: 10) {
      edges {
        node {
          sellingPlan {
            id
            name
            options {
              name
              value
            }
          }
          priceAdjustments {
            compareAtPrice {
              amount
              currencyCode
            }
            perDeliveryPrice {
              amount
              currencyCode
            }
            price {
              amount
              currencyCode
            }
          }
        }
      }
    }
  }
`,g=`
  fragment product on Product {
    id
    handle
    availableForSale
    title
    description
    descriptionHtml
    options {
      id
      name
      values
    }
    productType
    vendor
    priceRange {
      maxVariantPrice {
        amount
        currencyCode
      }
      minVariantPrice {
        amount
        currencyCode
      }
    }
    compareAtPriceRange {
      minVariantPrice {
        amount
        currencyCode
      }
    }
    featuredImage {
      ...image
    }
    images(first: 250) {
      edges {
        node {
          ...image
        }
      }
    }
    metafields(identifiers: $productMetafieldIdentifiers) {
      ...metafield
    }
    media(first: 250) {
      edges {
        node {
          id
          mediaContentType
          ... on Video {
            id
            sources {
              format
              height
              mimeType
              url
              width
            }
          }
          ... on MediaImage {
            id
            image {
              altText
              url
            }
          }
        }
      }
    }
    seo {
      ...seo
    }
    tags
    createdAt
    updatedAt
    publishedAt
    requiresSellingPlan
    sellingPlanGroups(first: 250) {
      edges {
        node {
          name
          sellingPlans(first: 250) {
            edges {
              node {
                id
                name
                priceAdjustments {
                  adjustmentValue {
                    ... on SellingPlanFixedAmountPriceAdjustment {
                      __typename
                      adjustmentAmount {
                        amount
                        currencyCode
                      }
                    }
                    ... on SellingPlanFixedPriceAdjustment {
                      __typename
                      price {
                        amount
                        currencyCode
                      }
                    }
                    ... on SellingPlanPercentagePriceAdjustment {
                      __typename
                      adjustmentPercentage
                    }
                  }
                }
                options {
                  name
                  value
                }
              }
            }
          }
        }
      }
    }
  }
`,y=`
  fragment cart on Cart {
    id
    totalQuantity
    updatedAt
    createdAt
    checkoutUrl
    note
    attributes {
      key
      value
    }
    cost {
      checkoutChargeAmount {
        amount
        currencyCode
      }
      subtotalAmount {
        amount
        currencyCode
      }
      totalAmount {
        amount
        currencyCode
      }
      totalTaxAmount {
        amount
        currencyCode
      }
      totalDutyAmount {
        amount
        currencyCode
      }
      totalAmountEstimated
      totalTaxAmountEstimated
      totalDutyAmountEstimated
      subtotalAmountEstimated
    }
    buyerIdentity {
      email
      phone
      countryCode
      customer {
        email
        id
      }
      deliveryAddressPreferences {
        ... on MailingAddress {
          address1
          address2
          city
          province
          country
          zip
          firstName
          lastName
        }
      }
    }
    discountCodes {
      applicable
      code
    }
    discountAllocations {
      ... on CartCodeDiscountAllocation {
        code
        discountedAmount {
          amount
          currencyCode
        }
      }
      ... on CartAutomaticDiscountAllocation {
        title
        discountedAmount {
          amount
          currencyCode
        }
      }
      ... on CartCustomDiscountAllocation {
        title
        discountedAmount {
          amount
          currencyCode
        }
      }
    }
    deliveryGroups(first: 10) {
      nodes {
        ... on CartDeliveryGroup {
          id
          deliveryOptions {
            ... on CartDeliveryOption {
              code
              description
              deliveryMethodType
              handle
              title
              estimatedCost {
                amount
                currencyCode
              }
            }
          }
          deliveryAddress {
            ... on MailingAddress {
              address1
              address2
              city
              country
              zip
            }
          }
          selectedDeliveryOption {
            ... on CartDeliveryOption {
              code
              description
              deliveryMethodType
              handle
              title
              estimatedCost {
                amount
                currencyCode
              }
            }
          }
        }
      }
    }
    lines(first: 100) {
      edges {
        node {
          id
          quantity
          discountAllocations {
            ... on CartCodeDiscountAllocation {
              code
              discountedAmount {
                amount
                currencyCode
              }
            }
            ... on CartAutomaticDiscountAllocation {
              title
              discountedAmount {
                amount
                currencyCode
              }
            }
            ... on CartCustomDiscountAllocation {
              title
              discountedAmount {
                amount
                currencyCode
              }
            }
          }
          attributes {
            key
            value
          }
          cost {
            amountPerQuantity {
              amount
              currencyCode
            }
            compareAtAmountPerQuantity {
              amount
              currencyCode
            }
            subtotalAmount {
              amount
              currencyCode
            }
            totalAmount {
              amount
              currencyCode
            }
          }
          merchandise {
            ... on ProductVariant {
              id
              sku
              title
              requiresShipping
              quantityAvailable
              currentlyNotInStock
              availableForSale
              barcode
              weight
              selectedOptions {
                name
                value
              }
              image {
                ...image
              }
              price {
                amount
                currencyCode
              }
              compareAtPrice {
                amount
                currencyCode
              }
              metafields(identifiers: $variantMetafieldIdentifiers) {
                ...metafield
              }
              product {
                ...product
              }
            }
          }
        }
      }
    }
    totalQuantity
  }
  ${g}
  ${c}
  ${f}
  ${m}
`,h=`
  fragment collection on Collection {
    id
    handle
    title
    description
    descriptionHtml
    image {
      ...image
    }
    seo {
      ...seo
    }
    updatedAt
  }
`,v=`
  fragment pageInfo on PageInfo {
    hasNextPage
    hasPreviousPage
    startCursor
    endCursor
  }
`,b=`
  fragment article on Article {
    id
    handle
    title
    content
    contentHtml
    excerpt
    excerptHtml
    publishedAt
    image {
      ...image
    }
    seo {
      ...seo
    }
    tags
    author {
      name
    }
  }
`,_=`
  fragment articleWithMetafields on Article {
    id
    handle
    title
    content
    contentHtml
    excerpt
    excerptHtml
    publishedAt
    image {
      ...image
    }
    seo {
      ...seo
    }
    tags
    author {
      name
    }
    metafields(identifiers: $articleMetafieldIdentifiers) {
      ...metafield
    }
  }
`,I=`
  fragment blog on Blog {
    id
    handle
    title
    seo {
      ...seo
    }
  }
`,C=`
  fragment blogWithMetafields on Blog {
    id
    handle
    title
    seo {
      ...seo
    }
    metafields(identifiers: $blogMetafieldIdentifiers) {
      ...metafield
    }
  }
`,A=`
  fragment shop on Shop {
    name
    description
    moneyFormat
    primaryDomain {
      host
      url
    }
    paymentSettings {
      currencyCode
      enabledPresentmentCurrencies
    }
    brand {
      logo {
        image {
          url
        }
      }
      colors {
        primary {
          background
        }
        secondary {
          background
        }
      }
    }
  }
`,w=`
  fragment shopWithMetafields on Shop {
    name
    description
    moneyFormat
    primaryDomain {
      host
      url
    }
    paymentSettings {
      currencyCode
      enabledPresentmentCurrencies
    }
    brand {
      logo {
        image {
          url
        }
      }
      colors {
        primary {
          background
        }
        secondary {
          background
        }
      }
    }
    metafields(identifiers: $shopMetafieldIdentifiers) {
      ...metafield
    }
  }
  ${m}
`,$=`
  query getCart(
    $cartId: ID!
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!]!
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!]!
  ) {
    cart(id: $cartId) {
      ...cart
    }
  }
  ${y}
`,P=`
  mutation createCart(
    $lines: [CartLineInput!]
    $discountCodes: [String!]
    $attributes: [AttributeInput!] = []
    $buyerIdentity: CartBuyerIdentityInput
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    cartCreate(
      input: {
        lines: $lines
        discountCodes: $discountCodes
        attributes: $attributes
        buyerIdentity: $buyerIdentity
      }
    ) {
      cart {
        ...cart
      }
      userErrors {
        field
        message
      }
    }
  }
  ${y}
`,k=`
  mutation addToCart(
    $cartId: ID!
    $lines: [CartLineInput!]!
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    cartLinesAdd(cartId: $cartId, lines: $lines) {
      cart {
        ...cart
      }
      userErrors {
        field
        message
      }
    }
  }
  ${y}
`,M=`
  mutation editCartItems(
    $cartId: ID!
    $lines: [CartLineUpdateInput!]!
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    cartLinesUpdate(cartId: $cartId, lines: $lines) {
      cart {
        ...cart
      }
      userErrors {
        field
        message
      }
    }
  }
  ${y}
`,S=`
  mutation removeFromCart(
    $cartId: ID!
    $lineIds: [ID!]!
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    cartLinesRemove(cartId: $cartId, lineIds: $lineIds) {
      cart {
        ...cart
      }
      userErrors {
        field
        message
      }
    }
  }
  ${y}
`,E=`
  mutation cartDiscountCodesUpdate(
    $cartId: ID!
    $discountCodes: [String!]!
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    cartDiscountCodesUpdate(cartId: $cartId, discountCodes: $discountCodes) {
      cart {
        ...cart
      }
      userErrors {
        field
        message
      }
    }
  }
  ${y}
`,N=`
  mutation cartAttributesUpdate(
    $attributes: [AttributeInput!]!
    $cartId: ID!
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    cartAttributesUpdate(cartId: $cartId, attributes: $attributes) {
      cart {
        ...cart
      }
      userErrors {
        field
        message
      }
    }
  }
  ${y}
`,T=`
  mutation cartBuyerIdentityUpdate(
    $buyerIdentity: CartBuyerIdentityInput!
    $cartId: ID!
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    cartBuyerIdentityUpdate(cartId: $cartId, buyerIdentity: $buyerIdentity) {
      cart {
        ...cart
      }
      userErrors {
        field
        message
      }
    }
  }
  ${y}
`;function q(e){let t=e.images?.edges?.map(e=>e.node)||[],r=e.media?.edges?.map(e=>e.node)||[],n=e.variants?.edges?.map(e=>{var t;let r,n,i,a,o;return t=e.node,r={amount:t.price?.amount||t.priceV2?.amount||"0",currencyCode:t.price?.currencyCode||t.priceV2?.currencyCode||"USD"},n=t.compareAtPrice?.amount?{amount:t.compareAtPrice.amount,currencyCode:t.compareAtPrice.currencyCode}:t.compareAtPriceV2?.amount?{amount:t.compareAtPriceV2.amount,currencyCode:t.compareAtPriceV2.currencyCode}:void 0,i=9999999.99===Number(r.amount),a=l(t.metafields||[]),{id:t.id,title:t.title,sku:t.sku||"",requiresShipping:t.requiresShipping??!0,availableForSale:!i&&(t.availableForSale||!1),quantityAvailable:t.quantityAvailable,selectedOptions:t.selectedOptions||[],price:r,compareAtPrice:n,image:t.image?t.image:void 0,weight:t.weight,weightUnit:t.weightUnit,metafields:a,sellingPlanAllocations:(o=t.sellingPlanAllocations,o?.edges?.map(({node:e})=>({...e}))||[])}})||[],i={amount:e.priceRange?.minVariantPrice?.amount||"0",currencyCode:e.priceRange?.minVariantPrice?.currencyCode||"USD"},a=e.compareAtPriceRange?.minVariantPrice?.amount?{amount:e.compareAtPriceRange.minVariantPrice.amount,currencyCode:e.compareAtPriceRange.minVariantPrice.currencyCode}:void 0,o=l(e.metafields||[]);return{id:e.id,handle:e.handle,title:e.title,description:e.description,descriptionHtml:e.descriptionHtml,availableForSale:e.availableForSale,tags:e.tags,productType:e.productType,vendor:e.vendor,price:i,compareAtPrice:a,images:t,media:r,featuredImage:e.featuredImage?e.featuredImage:void 0,variants:n,options:e.options,seo:e.seo,createdAt:e.createdAt,updatedAt:e.updatedAt,publishedAt:e.publishedAt,metafields:o,sellingPlanGroups:e.sellingPlanGroups?(({edges:e})=>e?.map(({node:e})=>{let{sellingPlans:t,...r}=e;return{...r,sellingPlans:(({edges:e})=>e?.map(({node:e})=>e))(t)}}))(e.sellingPlanGroups):[]}}var L=`
  query getProductByHandle(
    $handle: String!
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    product(handle: $handle) {
      ...product
      metafields(identifiers: $productMetafieldIdentifiers) {
        ...metafield
      }
      variants(first: 250) {
        edges {
          node {
            ...variant
            metafields(identifiers: $variantMetafieldIdentifiers) {
              ...metafield
            }
          }
        }
      }
    }
  }
  ${g}
  ${p}
  ${m}
  ${f}
  ${c}
`;async function D(e,t){let{handle:r,graphqlQuery:n,metafieldIdentifiers:i}=t,a=await e.query(n||L,{handle:r,...u(i,e.getConfig().getMetafieldNamespacePrefix())});if(a?.product)return q(a.product)}var O=`
  query getAllProducts(
    $first: Int = 250
    $after: String
    $query: String
    $sortKey: ProductSortKeys
    $reverse: Boolean
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    products(first: $first, after: $after, query: $query, sortKey: $sortKey, reverse: $reverse) {
      pageInfo {
        hasNextPage
        hasPreviousPage
        startCursor
        endCursor
      }
      edges {
        node {
          ...product
          metafields(identifiers: $productMetafieldIdentifiers) {
            ...metafield
          }
          variants(first: 250) {
            edges {
              node {
                ...variant
                metafields(identifiers: $variantMetafieldIdentifiers) {
                  ...metafield
                }
              }
            }
          }
        }
      }
    }
  }
  ${g}
  ${p}
  ${m}
  ${f}
  ${c}
`;async function R(e,t,r){let{first:n=250,query:i,graphqlQuery:a,sortKey:o,reverse:s,metafieldIdentifiers:l}=t,d=await e.query(a||O,{first:n,after:r,query:i,sortKey:o,reverse:s,...u(l,e.getConfig().getMetafieldNamespacePrefix())});if(!d||!d.products)return[];let c=d.products.edges.map(e=>q(e.node));return d.products.pageInfo.hasNextPage?[...c,...await R(e,t,d.products.pageInfo.endCursor)]:c}async function x(e,t){return R(e,t)}async function U(e,t){let{first:r=250,after:n,query:i,graphqlQuery:a,sortKey:o,reverse:u,metafieldIdentifiers:s}=t,l={first:r,after:n,query:i,sortKey:o,reverse:u};s&&(l.productMetafieldIdentifiers=s,l.variantMetafieldIdentifiers=s);let d=await e.query(a||O,l);return d&&d.products?{products:d.products.edges.map(e=>q(e.node)),pageInfo:d.products.pageInfo}:{products:[],pageInfo:{hasNextPage:!1,hasPreviousPage:!1}}}async function H(e,t){let{handles:r,graphqlQuery:n,metafieldIdentifiers:i}=t;if(!r||0===r.length)return[];let a=n||`
    query GetProductByHandles($productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = [], $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []) {
      ${r.map((e,t)=>`
          product_${t}: product(handle: "${e}") {
            ...product
            variants(first: 250) {
              edges {
                node {
                  ...variant
                }
              }
            }
          }
        `)}
    }
  ${g}
  ${p}
  ${m}
  ${f}
  ${c}
`,o=await e.query(a,{handles:r,...u(i,e.getConfig().getMetafieldNamespacePrefix())});return o?Object.values(o).map(e=>{if(e)return q(e)}):[]}function F(e){let{id:t,merchandise:r,quantity:n,discountAllocations:i,attributes:a,cost:o}=e,u=r?.price?.amount;return{id:t,name:r?.product?.title||"",quantity:n,variantId:String(r?.id||""),productId:String(r?.product?.id||""),totalAmount:o?.totalAmount?.amount,subtotalAmount:o?.subtotalAmount?.amount,discountAllocations:i?.map(e=>({title:e?.title||"",code:e?.code||"",amount:e?.discountedAmount?.amount||0}))||[],customAttributes:a?.filter(e=>e.key&&e.value).map(e=>({key:e.key,value:e.value||""}))||[],variant:{id:String(r?.id||""),price:u,listPrice:r?.compareAtPrice?.amount,sku:r?.sku??"",name:r?.title||"",image:r?.image?{url:r.image.url,altText:r.image.altText}:void 0,requiresShipping:r?.requiresShipping??!1,availableForSale:!(!r.availableForSale||!r.quantityAvailable&&!r.currentlyNotInStock||r.price?.amount>=9999999.99),quantityAvailable:r?.quantityAvailable||0,currentlyNotInStock:r?.currentlyNotInStock||!1,weight:r?.weight||0,metafields:l(r?.metafields)},amountPerQuantity:o?.amountPerQuantity?.amount,product:q(r?.product),path:`/${r?.product?.handle||""}`,options:r?.title==="Default Title"?[]:r?.selectedOptions||[]}}function B(e){let t=e.lines.edges.map(e=>e.node).map(F),r=e?.discountAllocations?.reduce((e,t)=>Number(t?.discountedAmount?.amount||0)+e,0)||0,[n,i]=t.reduce(([e,t],r)=>[(r.discountAllocations?.reduce((e,t)=>Number(t.amount)+e,0)||0)+e,Number(r.subtotalAmount)+t],[0,0]);return{id:e.id,url:e.checkoutUrl,ready:!0,createdAt:e.createdAt,currency:{code:e.cost?.subtotalAmount?.currencyCode||"USD"},customAttributes:e.attributes.filter(e=>e.key&&e.value).map(e=>({key:e.key,value:e.value||""}))||[],taxesIncluded:!1,lineItems:t,subtotalPrice:Number(e.cost?.subtotalAmount?.amount||0),totalPrice:Number(e?.cost?.totalAmount?.amount||0)-Number(e?.cost?.totalTaxAmount?.amount||0),totalTaxAmount:0,lineItemsSubtotalPrice:i,totalLineItemsDiscount:r+n,orderDiscounts:r,discountCodes:e.discountCodes||[],discountAllocations:e?.discountAllocations?.map(e=>({title:e?.title||"",code:e?.code||"",amount:Number(e?.discountedAmount?.amount||0)}))||[],deliveryAmount:e?.deliveryGroups?.nodes?.[0]?.selectedDeliveryOption?.estimatedCost,deliveryGroups:e?.deliveryGroups?.nodes,buyerIdentity:e?.buyerIdentity}}async function G(e,t){let{id:r,cookieAdapter:n,graphqlQuery:i,metafieldIdentifiers:a}=t,o=e.getLocale(),s=r||n?.getCartId(o);if(console.log("cartId",s),s)try{let t=await e.query(i||$,{cartId:s,...u(a,e.getConfig().getMetafieldNamespacePrefix())});if(!t?.cart){n&&!r&&n.removeCartId(o);return}let l=B(t.cart);return n&&l.id&&!r&&n.setCartId(o,l.id),l}catch(e){console.error("[getCart] Error fetching cart:",e),n&&!r&&n.removeCartId(o);return}}async function j(e,t={}){let{lines:r,cookieAdapter:n,buyerIdentity:i,discountCodes:a,customAttributes:o,graphqlQuery:s,metafieldIdentifiers:l,updateCookie:d=!1}=t,c=e.getLocale();try{let t=await e.query(s||P,{lines:r,buyerIdentity:i,discountCodes:a,attributes:o,...u(l,e.getConfig().getMetafieldNamespacePrefix())});t?.cartCreate?.userErrors?.length&&console.error("[createCart] User errors:",t.cartCreate.userErrors);let f=t?.cartCreate?.cart;if(!f)return;let m=B(f);return n&&m.id&&d&&n.setCartId(c,m.id),m}catch(e){console.error("[createCart] Error:",e);return}}async function V(e,t){let{cartId:r,lines:n,cookieAdapter:i,graphqlQuery:a,metafieldIdentifiers:o}=t,s=e.getLocale(),l=r||i?.getCartId(s);if(!l)throw Error("Invalid input used for this operation: Miss cartId");let d=u(o,e.getConfig().getMetafieldNamespacePrefix());try{let t=await e.query(a||k,{cartId:l,lines:n,...d});t?.cartLinesAdd?.userErrors?.length&&console.error("[addCartLines] User errors:",t.cartLinesAdd.userErrors);let r=t?.cartLinesAdd?.cart;if(!r)return;let o=B(r);return i&&o.id&&i.setCartId(s,o.id),o}catch(e){console.error("[addCartLines] Error:",e);return}}async function W(e,t){let{cartId:r,lines:n,cookieAdapter:i,graphqlQuery:a,metafieldIdentifiers:o}=t,s=e.getLocale(),l=r||i?.getCartId(s);if(!l)throw Error("Invalid input used for this operation: Miss cartId");let d=u(o,e.getConfig().getMetafieldNamespacePrefix());console.log("update-cart-lines metafieldIdentifiers:",o),console.log("update-cart-lines metafieldParams:",d);try{let t,r=await e.query(a||M,{cartId:l,lines:n,...d});if(r?.cartLinesUpdate?.userErrors?.length){let e=r.cartLinesUpdate.userErrors.find(e=>e.field?.includes("quantity"));if(e&&e.field&&void 0!==e.field[1]){let n=Number(e.field[1]),i=r.cartLinesUpdate.cart;t=i?.lines?.edges?.[n]?.node?.quantity}}if(!r?.cartLinesUpdate?.cart)return;let o=B(r.cartLinesUpdate.cart);return void 0!==t&&(o.maxNum=t),i&&o.id&&i.setCartId(s,o.id),o}catch(e){throw console.error("[updateCartLines] Error:",e),e}}async function J(e,t){let{cartId:r,lineIds:n,cookieAdapter:i,graphqlQuery:a,metafieldIdentifiers:o}=t,s=e.getLocale(),l=r||i?.getCartId(s);if(!l)throw Error("Invalid input used for this operation: Miss cartId");try{let t=await e.query(a||S,{cartId:l,lineIds:n,...u(o,e.getConfig().getMetafieldNamespacePrefix())});if(t?.cartLinesRemove?.userErrors?.length&&console.error("[removeCartLines] User errors:",t.cartLinesRemove.userErrors),!t?.cartLinesRemove?.cart)return;let r=B(t.cartLinesRemove.cart);return i&&r.id&&i.setCartId(s,r.id),r}catch(e){throw console.error("[removeCartLines] Error:",e),e}}async function K(e,t){let{cartId:r,discountCodes:n,cookieAdapter:i,graphqlQuery:a,metafieldIdentifiers:o}=t,s=e.getLocale(),l=r||i?.getCartId(s);if(!l)throw Error("Invalid input used for this operation: Miss cartId");try{let t=await e.query(a||E,{cartId:l,discountCodes:n,...u(o,e.getConfig().getMetafieldNamespacePrefix())});if(t?.cartDiscountCodesUpdate?.userErrors?.length&&console.error("[updateCartCodes] User errors:",t.cartDiscountCodesUpdate.userErrors),!t?.cartDiscountCodesUpdate?.cart)return;return B(t.cartDiscountCodesUpdate.cart)}catch(e){throw console.error("[updateCartCodes] Error:",e),e}}async function Q(e,t){let{cartId:r,attributes:n,cookieAdapter:i,graphqlQuery:a,metafieldIdentifiers:o}=t,s=e.getLocale(),l=r||i?.getCartId(s);if(!l)throw Error("Invalid input used for this operation: Miss cartId");try{let t=await e.query(a||N,{cartId:l,attributes:n,...u(o,e.getConfig().getMetafieldNamespacePrefix())});if(t?.cartAttributesUpdate?.userErrors?.length&&console.error("[updateCartAttributes] User errors:",t.cartAttributesUpdate.userErrors),!t?.cartAttributesUpdate?.cart)return;let r=B(t.cartAttributesUpdate.cart);return i&&r.id&&i.setCartId(s,r.id),r}catch(e){throw console.error("[updateCartAttributes] Error:",e),e}}async function Y(e,t){let{cartId:r,buyerIdentity:n,cookieAdapter:i,graphqlQuery:a,metafieldIdentifiers:o}=t,s=e.getLocale(),l=r||i?.getCartId(s);if(!l)throw Error("Invalid input used for this operation: Miss cartId");try{let t=await e.query(a||T,{cartId:l,buyerIdentity:n,...u(o,e.getConfig().getMetafieldNamespacePrefix())});if(t?.cartBuyerIdentityUpdate?.userErrors?.length&&console.error("[updateBuyerIdentity] User errors:",t.cartBuyerIdentityUpdate.userErrors),!t?.cartBuyerIdentityUpdate?.cart)return;let r=B(t.cartBuyerIdentityUpdate.cart);return i&&r.id&&i.setCartId(s,r.id),r}catch(e){throw console.error("[updateBuyerIdentity] Error:",e),e}}function z(e){let t=e.products?.edges?.map(e=>q(e.node)),r={};return e.metafields?.edges&&e.metafields.edges.forEach(e=>{r[e.node.key]=e.node.value}),{id:e.id,handle:e.handle,title:e.title,description:e.description,descriptionHtml:e.descriptionHtml,image:e.image,products:t,seo:e.seo,updatedAt:e.updatedAt,metafields:Object.keys(r).length>0?r:void 0}}var X=`
  query getCollectionByHandle(
    $handle: String!
    $metafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    collection(handle: $handle) {
      ...collection
      metafields(identifiers: $metafieldIdentifiers) {
        ...metafield
      }
    }
  }
  ${h}
  ${m}
  ${f}
  ${c}
`;async function Z(e,t){let{handle:r,graphqlQuery:n,metafieldIdentifiers:i}=t,a=await e.query(n||X,{handle:r,...u(i,e.getConfig().getMetafieldNamespacePrefix())});if(a?.collection)return z(a.collection)}var ee=`
  query getAllCollections(
    $first: Int = 250
    $after: String
    $query: String
    $sortKey: CollectionSortKeys
    $reverse: Boolean
    $metafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    collections(first: $first, after: $after, query: $query, sortKey: $sortKey, reverse: $reverse) {
      pageInfo {
        hasNextPage
        hasPreviousPage
        startCursor
        endCursor
      }
      edges {
        node {
          ...collection
          metafields(identifiers: $metafieldIdentifiers) {
            ...metafield
          }
        }
      }
    }
  }
  ${h}
  ${m}
  ${f}
  ${c}
`;async function et(e,t,r){let{first:n=250,query:i,graphqlQuery:a,sortKey:o,reverse:s,metafieldIdentifiers:l}=t,d=await e.query(a||ee,{first:n,after:r,query:i,sortKey:o,reverse:s,...u(l,e.getConfig().getMetafieldNamespacePrefix())});if(!d||!d.collections)return[];let c=d.collections.edges.map(e=>z(e.node));return d.collections.pageInfo.hasNextPage?[...c,...await et(e,t,d.collections.pageInfo.endCursor)]:c}async function er(e,t){return et(e,t)}async function en(e,t){let{first:r=250,after:n,query:i,graphqlQuery:a,sortKey:o,reverse:s,metafieldIdentifiers:l}=t,d=await e.query(a||ee,{first:r,after:n,query:i,sortKey:o,reverse:s,...u(l,e.getConfig().getMetafieldNamespacePrefix())});return d&&d.collections?{collections:d.collections.edges.map(e=>z(e.node)),pageInfo:d.collections.pageInfo}:{collections:[],pageInfo:{hasNextPage:!1,hasPreviousPage:!1}}}async function ei(e,t){let{handles:r,graphqlQuery:n,metafieldIdentifiers:i}=t;if(0===r.length)return[];let a=n||`
  ${h}
  ${g}
  ${p}
  ${m}
  ${c}
  ${f}

  query GetCollectionsByHandles(
    $collectionMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $productMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
    $variantMetafieldIdentifiers: [HasMetafieldsIdentifier!] = []
  ) {
    ${r.map((e,t)=>`
      collection_${t}: collection(handle: "${e}") {
        ...collection
        metafields(identifiers: $collectionMetafieldIdentifiers) {
          ...metafield
        }
        products(first: 9) {
          edges { 
            node {
              ...product
              metafields(identifiers: $productMetafieldIdentifiers) {
                ...metafield
              }
              variants(first: 10) {
                edges {
                  node {
                    ...variant
                    metafields(identifiers: $variantMetafieldIdentifiers) {
                      ...metafield
                    }
                  }
                }
              }
            }
          }
        }
      }
    `).join("\n")}
  }
`,o=await e.query(a,{handles:r,...u(i,e.getConfig().getMetafieldNamespacePrefix())});return o?Object.values(o).map(e=>{if(e)return z(e)}):[]}function ea(e){return{id:e.id,handle:e.handle,title:e.title,seo:e.seo,metafields:eu(e.metafields),articles:e.articles?.edges?.map(e=>eo(e.node))||[]}}function eo(e){return{id:e.id,handle:e.handle,title:e.title,content:e.content,contentHtml:e.contentHtml,excerpt:e.excerpt,excerptHtml:e.excerptHtml,publishedAt:e.publishedAt,updatedAt:e.updatedAt,image:e.image,seo:e.seo,tags:e.tags||[],author:e.author,blog:e.blog,metafields:eu(e.metafields)}}function eu(e){if(e&&0!==e.length)return e.reduce((e,t)=>(t&&t.key&&(e[t.key]=t.value),e),{})}async function es(e,t){let{handle:r,graphqlQuery:n,metafieldIdentifiers:i}=t,a=i?.blog?.length,o=`
    query getBlog(
      $handle: String!
      ${a?"$blogMetafieldIdentifiers: [HasMetafieldsIdentifier!]!":""}
    ) {
      blog(handle: $handle) {
        ...${a?"blogWithMetafields":"blog"}
      }
    }
    ${a?m:""}
    ${f}
    ${a?C:I}
  `,s=await e.query(n||o,{handle:r,...u(i,e.getConfig().getMetafieldNamespacePrefix())});if(s&&s.blog)return ea(s.blog)}async function el(e,t,r){let{first:n=250,query:i,graphqlQuery:a,metafieldIdentifiers:o}=t,s=!!o?.blog?.length,l=`
    query getAllBlogs(
      $first: Int!
      $after: String
      ${i?"$query: String!":""}
      ${s?"$blogMetafieldIdentifiers: [HasMetafieldsIdentifier!]!":""}
    ) {
      blogs(
        first: $first
        after: $after
        ${i?"query: $query":""}
      ) {
        edges {
          node {
            ...${s?"blogWithMetafields":"blog"}
          }
        }
        pageInfo {
          ...pageInfo
        }
      }
    }
    ${s?m:""}
    ${f}
    ${s?C:I}
    ${v}
  `,d={first:n,after:r};i&&(d.query=i);let c=await e.query(a||l,{...d,...u(o,e.getConfig().getMetafieldNamespacePrefix())});if(!c||!c.blogs)return[];let p=c.blogs.edges?.map(e=>ea(e.node))||[];return c.blogs.pageInfo.hasNextPage?[...p,...await el(e,t,c.blogs.pageInfo.endCursor)]:p}async function ed(e,t){return el(e,t)}async function ec(e,t){let{blogHandle:r,articleHandle:n,graphqlQuery:i,metafieldIdentifiers:a}=t,o=!!a?.article?.length,s=!!a?.blog?.length,l=`
    query getArticle(
      $blogHandle: String!
      $articleHandle: String!
      ${s?"$blogMetafieldIdentifiers: [HasMetafieldsIdentifier!]!":""}
      ${o?"$articleMetafieldIdentifiers: [HasMetafieldsIdentifier!]!":""}
    ) {
      blog(handle: $blogHandle) {
        ...${s?"blogWithMetafields":"blog"}
        articleByHandle(handle: $articleHandle) {
          ...${o?"articleWithMetafields":"article"}
          blog {
            ...${s?"blogWithMetafields":"blog"}
          }
        }
      }
    }
    ${c}
    ${o||s?m:""}
    ${f}
    ${o?_:b}
    ${s?C:I}
  `,d=await e.query(i||l,{blogHandle:r,articleHandle:n,...u(a,e.getConfig().getMetafieldNamespacePrefix())});if(d&&d.blog&&d.blog.articleByHandle)return eo(d.blog.articleByHandle)}async function ef(e,t,r){let{first:n=250,query:i,graphqlQuery:a,sortKey:o="PUBLISHED_AT",reverse:s=!1,metafieldIdentifiers:l}=t,d=!!l?.article?.length,p=`
    query getArticles(
      $first: Int!
      $after: String
      ${i?"$query: String!":""}
      $sortKey: ArticleSortKeys!
      $reverse: Boolean!
      ${d?"$articleMetafieldIdentifiers: [HasMetafieldsIdentifier!]!":""}
    ) {
      articles(
        first: $first
        after: $after
        ${i?"query: $query":""}
        sortKey: $sortKey
        reverse: $reverse
      ) {
        edges {
          node {
            ...${d?"articleWithMetafields":"article"}
          }
        }
        pageInfo {
          ...pageInfo
        }
      }
    }
    ${c}
    ${d?m:""}
    ${f}
    ${d?_:b}
    ${v}
  `,g={first:n,after:r,sortKey:o,reverse:s};i&&(g.query=i);let y=await e.query(a||p,{...g,...u(l,e.getConfig().getMetafieldNamespacePrefix())});if(!y||!y.articles)return[];let h=y.articles.edges?.map(e=>eo(e.node))||[];return y.articles.pageInfo.hasNextPage?[...h,...await ef(e,t,y.articles.pageInfo.endCursor)]:h}async function em(e,t){return ef(e,t)}async function ep(e,t,r){let{blogHandle:n,first:i=250,graphqlQuery:a,sortKey:o="PUBLISHED_AT",reverse:s=!1,metafieldIdentifiers:l}=t,d=!!l?.article?.length,p=!!l?.blog?.length,g=`
    query getArticlesInBlog(
      $blogHandle: String!
      $first: Int!
      $after: String
      $sortKey: ArticleSortKeys!
      $reverse: Boolean!
      ${p?"$blogMetafieldIdentifiers: [HasMetafieldsIdentifier!]!":""}
      ${d?"$articleMetafieldIdentifiers: [HasMetafieldsIdentifier!]!":""}
    ) {
      blog(handle: $blogHandle) {
        ...${p?"blogWithMetafields":"blog"}
        articles(
          first: $first
          after: $after
          sortKey: $sortKey
          reverse: $reverse
        ) {
          edges {
            node {
              ...${d?"articleWithMetafields":"article"}
            }
          }
          pageInfo {
            ...pageInfo
          }
        }
      }
    }
    ${c}
    ${d||p?m:""}
    ${f}
    ${d?_:b}
    ${p?C:I}
    ${v}
  `,y=await e.query(a||g,{blogHandle:n,first:i,after:r,sortKey:o,reverse:s,...u(l,e.getConfig().getMetafieldNamespacePrefix())});return y&&y.blog&&y.blog.articles&&y.blog.articles.edges?.map(e=>eo(e.node))||[]}async function eg(e,t){return ep(e,t)}async function ey(e,t){let{graphqlQuery:r,metafieldIdentifiers:n}=t,i=!!n?.shop?.length,a=`
    query getShop(
      ${i?"$shopMetafieldIdentifiers: [HasMetafieldsIdentifier!]!":""}
    ) {
      shop {
        ...${i?"shopWithMetafields":"shop"}
      }
    }
    ${i?w:A}
  `,o=await e.query(r||a,{...u(n,e.getConfig().getMetafieldNamespacePrefix())});if(o&&o.shop){var s;return{name:(s=o.shop).name,description:s.description||"",moneyFormat:s.moneyFormat,paymentSettings:s.paymentSettings,primaryDomain:s.primaryDomain,brand:s.brand||void 0,metafields:l(s.metafields)}}}e.s(["addCartLines",0,V,"createCart",0,j,"createShopifyClient",0,function(e,t){return new d(e,t)},"getAllBlogs",0,ed,"getAllCollections",0,er,"getAllProducts",0,x,"getArticle",0,ec,"getArticles",0,em,"getArticlesInBlog",0,eg,"getBlog",0,es,"getCart",0,G,"getCollection",0,Z,"getCollections",0,en,"getCollectionsByHandles",0,ei,"getProduct",0,D,"getProducts",0,U,"getProductsByHandles",0,H,"getShop",0,ey,"removeCartLines",0,J,"updateBuyerIdentity",0,Y,"updateCartAttributes",0,Q,"updateCartCodes",0,K,"updateCartLines",0,W],174898)},226536,e=>{"use strict";var t=e.i(564996),r=e.i(414120),n=e.i(558314),i=e.i(981423);let a=n.j?e=>{e()}:t.default.startTransition,o=(0,i.withMiddleware)(r.u,()=>(e,r,o={})=>{let{mutate:u}=(0,i.useSWRConfig)(),s=(0,t.useRef)(e),l=(0,t.useRef)(r),d=(0,t.useRef)(o),c=(0,t.useRef)(0),[f,m,p]=(e=>{let[,r]=(0,t.useState)({}),i=(0,t.useRef)(!1),a=(0,t.useRef)(e),o=(0,t.useRef)({data:!1,error:!1,isValidating:!1}),u=(0,t.useCallback)(e=>{let t=!1,n=a.current;for(let r in e)Object.prototype.hasOwnProperty.call(e,r)&&n[r]!==e[r]&&(n[r]=e[r],o.current[r]&&(t=!0));t&&!i.current&&r({})},[]);return(0,n.u)(()=>(i.current=!1,()=>{i.current=!0})),[a,o.current,u]})({data:n.U,error:n.U,isMutating:!1}),g=f.current,y=(0,t.useCallback)(async(e,t)=>{let[r,i]=(0,n.s)(s.current);if(!l.current)throw Error("Can’t trigger the mutation: missing fetcher.");if(!r)throw Error("Can’t trigger the mutation: missing key.");let o=(0,n.m)((0,n.m)({populateCache:!1,throwOnError:!0},d.current),t),f=(0,n.g)();c.current=f,p({isMutating:!0});try{let t=await u(r,l.current(i,{arg:e}),(0,n.m)(o,{throwOnError:!0}));return c.current<=f&&(a(()=>p({data:t,isMutating:!1,error:void 0})),null==o.onSuccess||o.onSuccess.call(o,t,r,o)),t}catch(e){if(c.current<=f&&(a(()=>p({error:e,isMutating:!1})),null==o.onError||o.onError.call(o,e,r,o),o.throwOnError))throw e}},[]),h=(0,t.useCallback)(()=>{c.current=(0,n.g)(),p({data:n.U,error:n.U,isMutating:!1})},[]);return(0,n.u)(()=>{s.current=e,l.current=r,d.current=o}),{trigger:y,reset:h,get data(){return m.data=!0,g.data},get error(){return m.error=!0,g.error},get isMutating(){return m.isMutating=!0,g.isMutating}}});e.s(["default",0,o])},850024,e=>{"use strict";var t=e.i(564996),r=e.i(489129),n=e.i(203590);e.s(["default",0,function(e){n.default&&!(0,r.isFunction)(e)&&console.error('useMount: parameter `fn` expected to be a function, but got "'.concat(typeof e,'".')),(0,t.useEffect)(function(){var t=null==e?void 0:e();if(!t||"object"!=typeof t||"function"!=typeof t.then)return t},[])}])},141376,133418,766011,280712,90328,e=>{"use strict";var t,r,n,i,a,o,u,s,l,d,c,f,m=e.i(564996),p=e.i(174898),g=e.i(385065),y=e.i(734804),h=e.i(134286),v=e.i(958083),b=e.i(667110),_=e.i(226536),I=e.i(824627),C=e.i(787331),C=C,A=e.i(421678),w=e.i(381378),$=e.i(489129),P=e.i(203590);let k=function(e,t){P.default&&!(0,$.isFunction)(e)&&console.error("useDebounceFn expected parameter is a function, got ".concat(typeof e));var r,n=(0,A.default)(e),i=null!=(r=null==t?void 0:t.wait)?r:1e3,a=(0,m.useMemo)(function(){return(0,C.default)(function(){for(var e=[],t=0;t<arguments.length;t++)e[t]=arguments[t];return n.current.apply(n,(0,I.__spreadArray)([],(0,I.__read)(e),!1))},i,t)},[]);return(0,w.default)(function(){a.cancel()}),{run:a,cancel:a.cancel,flush:a.flush}};e.s(["default",0,k],133418);let M=(t=m.useEffect,function(e,r){var n=(0,m.useRef)(!1);t(function(){return function(){n.current=!1}},[]),t(function(){if(n.current)return e();n.current=!0},r)}),S=function(e,t,r){var n=(0,I.__read)((0,m.useState)({}),2),i=n[0],a=n[1],o=k(function(){a({})},r).run;(0,m.useEffect)(function(){return o()},t),M(e,[i])};e.s(["useDebounceEffect",0,S],766011);var E=function(e,t){var r=t.manual,n=t.ready,i=void 0===n||n,a=t.defaultParams,o=void 0===a?[]:a,u=t.refreshDeps,s=t.refreshDepsAction,l=(0,m.useRef)(!1);return l.current=!1,M(function(){!r&&i&&(l.current=!0,e.run.apply(e,(0,I.__spreadArray)([],(0,I.__read)(o),!1)))},[i]),M(function(){!l.current&&(r||(l.current=!0,s?s():e.refresh()))},(0,I.__spreadArray)([],(0,I.__read)(void 0===u?[]:u),!1)),{onBefore:function(){if(!i)return{stopNow:!0}}}};E.onInit=function(e){var t=e.ready;return{loading:!e.manual&&(void 0===t||t)}};let N=function(e,t){if(e===t)return!0;for(var r=0;r<e.length;r++)if(!Object.is(e[r],t[r]))return!1;return!0};e.s(["default",0,N],280712);let T=function(e,t){var r=(0,m.useRef)({deps:t,obj:void 0,initialized:!1}).current;return!1!==r.initialized&&N(r.deps,t)||(r.deps=t,r.obj=e(),r.initialized=!0),r.obj};var q=new Map,L=function(e,t,r){var n=q.get(e);(null==n?void 0:n.timer)&&clearTimeout(n.timer);var i=void 0;t>-1&&(i=setTimeout(function(){q.delete(e)},t)),q.set(e,(0,I.__assign)((0,I.__assign)({},r),{timer:i}))},D=new Map,O=function(e,t){D.set(e,t),t.then(function(t){return D.delete(e),t}).catch(function(){D.delete(e)})},R={},x=function(e,t){R[e]&&R[e].forEach(function(e){return e(t)})},U=function(e,t){return R[e]||(R[e]=[]),R[e].push(t),function(){var r=R[e].indexOf(t);R[e].splice(r,1)}};let H=function(e,t){var r=t.cacheKey,n=t.cacheTime,i=void 0===n?3e5:n,a=t.staleTime,o=void 0===a?0:a,u=t.setCache,s=t.getCache,l=(0,m.useRef)(void 0),d=(0,m.useRef)(void 0),c=function(e,t){u?u(t):L(e,i,t),x(e,t.data)},f=function(e,t){return(void 0===t&&(t=[]),s)?s(t):q.get(e)};return(T(function(){if(r){var t=f(r);t&&Object.hasOwnProperty.call(t,"data")&&(e.state.data=t.data,e.state.params=t.params,(-1===o||Date.now()-t.time<=o)&&(e.state.loading=!1)),l.current=U(r,function(t){e.setState({data:t})})}},[]),(0,w.default)(function(){var e;null==(e=l.current)||e.call(l)}),r)?{onBefore:function(e){var t=f(r,e);return t&&Object.hasOwnProperty.call(t,"data")?-1===o||Date.now()-t.time<=o?{loading:!1,data:null==t?void 0:t.data,error:void 0,returnNow:!0}:{data:null==t?void 0:t.data,error:void 0}:{}},onRequest:function(e,t){var n=D.get(r);return n&&n!==d.current||(d.current=n=e.apply(void 0,(0,I.__spreadArray)([],(0,I.__read)(t),!1)),O(r,n)),{servicePromise:n}},onSuccess:function(t,n){var i;r&&(null==(i=l.current)||i.call(l),c(r,{data:t,params:n,time:Date.now()}),l.current=U(r,function(t){e.setState({data:t})}))},onMutate:function(t){var n;r&&(null==(n=l.current)||n.call(l),c(r,{data:t,params:e.state.params,time:Date.now()}),l.current=U(r,function(t){e.setState({data:t})}))}}:{}};var F=C;let B=function(e,t){var r=t.debounceWait,n=t.debounceLeading,i=t.debounceTrailing,a=t.debounceMaxWait,o=(0,m.useRef)(void 0),u=(0,m.useMemo)(function(){var e={};return void 0!==n&&(e.leading=n),void 0!==i&&(e.trailing=i),void 0!==a&&(e.maxWait=a),e},[n,i,a]);return((0,m.useEffect)(function(){if(r){var t=e.runAsync.bind(e);return o.current=(0,F.default)(function(e){e()},r,u),e.runAsync=function(){for(var e=[],r=0;r<arguments.length;r++)e[r]=arguments[r];return new Promise(function(r,n){var i;null==(i=o.current)||i.call(o,function(){t.apply(void 0,(0,I.__spreadArray)([],(0,I.__read)(e),!1)).then(r).catch(n)})})},function(){var r;null==(r=o.current)||r.cancel(),e.runAsync=t}}},[r,u]),r)?{onCancel:function(){var e;null==(e=o.current)||e.cancel()}}:{}},G=function(e,t){var r=t.loadingDelay,n=t.ready,i=(0,m.useRef)(void 0);if(!r)return{};var a=function(){i.current&&clearTimeout(i.current)};return{onBefore:function(){return a(),!1!==n&&(i.current=setTimeout(function(){e.setState({loading:!0})},r)),{loading:!1}},onFinally:function(){a()},onCancel:function(){a()}}};var j=!!("u">typeof window&&window.document&&window.document.createElement);function V(){return!j||"hidden"!==document.visibilityState}e.s(["default",0,j],90328);var W=new Set;j&&window.addEventListener("visibilitychange",function(){V()&&W.forEach(function(e){return e()})},!1);let J=function(e,t){var r=t.pollingInterval,n=t.pollingWhenHidden,i=void 0===n||n,a=t.pollingErrorRetryCount,o=void 0===a?-1:a,u=(0,m.useRef)(void 0),s=(0,m.useRef)(void 0),l=(0,m.useRef)(0),d=function(){var e;u.current&&clearTimeout(u.current),null==(e=s.current)||e.call(s)};return(M(function(){r||d()},[r]),r)?{onBefore:function(){d()},onError:function(){l.current+=1},onSuccess:function(){l.current=0},onFinally:function(){-1===o||-1!==o&&l.current<=o?u.current=setTimeout(function(){if(i||V())e.refresh();else{var t;t=function(){e.refresh()},W.add(t),s.current=function(){W.has(t)&&W.delete(t)}}},r):l.current=0},onCancel:function(){d()}}:{}};var K=new Set;if(j){var Q=function(){V()&&(!j||void 0===navigator.onLine||navigator.onLine)&&K.forEach(function(e){return e()})};window.addEventListener("visibilitychange",Q,!1),window.addEventListener("focus",Q,!1)}let Y=function(e,t){var r=t.refreshOnWindowFocus,n=t.focusTimespan,i=void 0===n?5e3:n,a=(0,m.useRef)(void 0),o=function(){var e;null==(e=a.current)||e.call(a)};return(0,m.useEffect)(function(){if(r){var t,n,u,s=(t=e.refresh.bind(e),n=!1,function(){for(var e=[],r=0;r<arguments.length;r++)e[r]=arguments[r];n||(n=!0,t.apply(void 0,(0,I.__spreadArray)([],(0,I.__read)(e),!1)),setTimeout(function(){n=!1},i))});u=function(){s()},K.add(u),a.current=function(){K.has(u)&&K.delete(u)}}return function(){o()}},[r,i]),(0,w.default)(function(){o()}),{}},z=function(e,t){var r=t.retryInterval,n=t.retryCount,i=(0,m.useRef)(void 0),a=(0,m.useRef)(0),o=(0,m.useRef)(!1);return n?{onBefore:function(){o.current||(a.current=0),o.current=!1,i.current&&clearTimeout(i.current)},onSuccess:function(){a.current=0},onError:function(){a.current+=1,-1===n||a.current<=n?i.current=setTimeout(function(){o.current=!0,e.refresh()},null!=r?r:Math.min(1e3*Math.pow(2,a.current),3e4)):a.current=0},onCancel:function(){a.current=0,i.current&&clearTimeout(i.current)}}:{}};var X=e.i(648250);let Z=function(e,t){var r=t.throttleWait,n=t.throttleLeading,i=t.throttleTrailing,a=(0,m.useRef)(void 0),o={};return(void 0!==n&&(o.leading=n),void 0!==i&&(o.trailing=i),(0,m.useEffect)(function(){if(r){var t=e.runAsync.bind(e);return a.current=(0,X.default)(function(e){e()},r,o),e.runAsync=function(){for(var e=[],r=0;r<arguments.length;r++)e[r]=arguments[r];return new Promise(function(r,n){var i;null==(i=a.current)||i.call(a,function(){t.apply(void 0,(0,I.__spreadArray)([],(0,I.__read)(e),!1)).then(r).catch(n)})})},function(){var r;e.runAsync=t,null==(r=a.current)||r.cancel()}}},[r,n,i]),r)?{onCancel:function(){var e;null==(e=a.current)||e.cancel()}}:{}},ee=function(e){P.default&&!(0,$.isFunction)(e)&&console.error("useMemoizedFn expected parameter is a function, got ".concat(typeof e));var t=(0,m.useRef)(e);t.current=(0,m.useMemo)(function(){return e},[e]);var r=(0,m.useRef)(void 0);return r.current||(r.current=function(){for(var e=[],r=0;r<arguments.length;r++)e[r]=arguments[r];return t.current.apply(this,e)}),r.current};var et=e.i(850024);let er=function(){var e=(0,I.__read)((0,m.useState)({}),2)[1];return ee(function(){return e({})})};var en=function(){function e(e,t,r,n){void 0===n&&(n={}),this.serviceRef=e,this.options=t,this.subscribe=r,this.initState=n,this.pluginImpls=[],this.count=0,this.state={loading:!1,params:void 0,data:void 0,error:void 0},this.state=(0,I.__assign)((0,I.__assign)((0,I.__assign)({},this.state),{loading:!t.manual}),n)}return e.prototype.setState=function(e){void 0===e&&(e={}),this.state=(0,I.__assign)((0,I.__assign)({},this.state),e),this.subscribe()},e.prototype.runPluginHandler=function(e){for(var t=[],r=1;r<arguments.length;r++)t[r-1]=arguments[r];var n=this.pluginImpls.map(function(r){var n;return null==(n=r[e])?void 0:n.call.apply(n,(0,I.__spreadArray)([r],(0,I.__read)(t),!1))}).filter(Boolean);return Object.assign.apply(Object,(0,I.__spreadArray)([{}],(0,I.__read)(n),!1))},e.prototype.runAsync=function(){for(var e=[],t=0;t<arguments.length;t++)e[t]=arguments[t];return(0,I.__awaiter)(this,void 0,void 0,function(){var t,r,n,i,a,o,u,s,l,d,c,f,m,p,g,y,h,v,b,_,C;return(0,I.__generator)(this,function(A){switch(A.label){case 0:if(this.count+=1,t=this.count,i=void 0!==(n=(r=this.runPluginHandler("onBefore",e)).stopNow)&&n,o=void 0!==(a=r.returnNow)&&a,u=(0,I.__rest)(r,["stopNow","returnNow"]),i||(this.setState((0,I.__assign)({loading:!0,params:e},u)),o))return[2,Promise.resolve(u.data)];null==(m=(f=this.options).onBefore)||m.call(f,e),A.label=1;case 1:return A.trys.push([1,3,,4]),(s=this.runPluginHandler("onRequest",this.serviceRef.current,e).servicePromise)||(s=(c=this.serviceRef).current.apply(c,(0,I.__spreadArray)([],(0,I.__read)(e),!1))),[4,s];case 2:if(l=A.sent(),t!==this.count)return[2,new Promise(function(){})];return this.setState({data:l,error:void 0,loading:!1}),null==(g=(p=this.options).onSuccess)||g.call(p,l,e),this.runPluginHandler("onSuccess",l,e),null==(h=(y=this.options).onFinally)||h.call(y,e,l,void 0),t===this.count&&this.runPluginHandler("onFinally",e,l,void 0),[2,l];case 3:if(d=A.sent(),t!==this.count)return[2,new Promise(function(){})];throw this.setState({error:d,loading:!1}),null==(b=(v=this.options).onError)||b.call(v,d,e),this.runPluginHandler("onError",d,e),null==(C=(_=this.options).onFinally)||C.call(_,e,void 0,d),t===this.count&&this.runPluginHandler("onFinally",e,void 0,d),d;case 4:return[2]}})})},e.prototype.run=function(){for(var e=this,t=[],r=0;r<arguments.length;r++)t[r]=arguments[r];this.runAsync.apply(this,(0,I.__spreadArray)([],(0,I.__read)(t),!1)).catch(function(t){e.options.onError||console.error(t)})},e.prototype.cancel=function(){this.count+=1,this.setState({loading:!1}),this.runPluginHandler("onCancel")},e.prototype.refresh=function(){this.run.apply(this,(0,I.__spreadArray)([],(0,I.__read)(this.state.params||[]),!1))},e.prototype.refreshAsync=function(){return this.runAsync.apply(this,(0,I.__spreadArray)([],(0,I.__read)(this.state.params||[]),!1))},e.prototype.mutate=function(e){var t=(0,$.isFunction)(e)?e(this.state.data):e;this.runPluginHandler("onMutate",t),this.setState({data:t})},e}();let ei=function(e,t,r){void 0===t&&(t={}),void 0===r&&(r=[]);var n=t.manual,i=void 0!==n&&n,a=t.ready,o=void 0===a||a,u=(0,I.__rest)(t,["manual","ready"]);P.default&&t.defaultParams&&!Array.isArray(t.defaultParams)&&console.warn("expected defaultParams is array, got ".concat(typeof t.defaultParams));var s=(0,I.__assign)({manual:i,ready:o},u),l=(0,A.default)(e),d=er(),c=T(function(){var e=r.map(function(e){var t;return null==(t=null==e?void 0:e.onInit)?void 0:t.call(e,s)}).filter(Boolean);return new en(l,s,d,Object.assign.apply(Object,(0,I.__spreadArray)([{}],(0,I.__read)(e),!1)))},[]);return c.options=s,c.pluginImpls=r.map(function(e){return e(c,s)}),(0,et.default)(function(){if(!i&&o){var e=c.state.params||t.defaultParams||[];c.run.apply(c,(0,I.__spreadArray)([],(0,I.__read)(e),!1))}}),(0,w.default)(function(){c.cancel()}),{loading:c.state.loading,data:c.state.data,error:c.state.error,params:c.state.params||[],cancel:ee(c.cancel.bind(c)),refresh:ee(c.refresh.bind(c)),refreshAsync:ee(c.refreshAsync.bind(c)),run:ee(c.run.bind(c)),runAsync:ee(c.runAsync.bind(c)),mutate:ee(c.mutate.bind(c))}},ea=function(e,t,r){return ei(e,t,(0,I.__spreadArray)((0,I.__spreadArray)([],(0,I.__read)(r||[]),!1),[B,G,J,Y,Z,E,H,z],!1))};var eo=(0,m.createContext)(null),eu={get:e=>y.default.get(e),set(e,t,r){y.default.set(e,t,{domain:r?.domain,path:r?.path||"/",expires:r?.expires,secure:r?.secure,sameSite:r?.sameSite})},remove(e){y.default.remove(e)}};function es(e){return`${e}-shopify-cart-id`}var el={getCartId:e=>y.default.get(es(e)),setCartId(e,t){y.default.set(es(e),t,{path:"/",expires:30})},removeCartId(e){y.default.remove(es(e))}};function ed(){let e=(0,m.useContext)(eo);if(!e)throw Error("useShopify must be used within a ShopifyProvider");return e}var ec=((r=ec||{})[r.AUTO_FREE_GIFT=1]="AUTO_FREE_GIFT",r[r.BUNDLE=2]="BUNDLE",r[r.VOLUME_DISCOUNT=3]="VOLUME_DISCOUNT",r[r.ORDER_DISCOUNT=4]="ORDER_DISCOUNT",r[r.PRICE_DISCOUNT=5]="PRICE_DISCOUNT",r),ef=((n=ef||{})[n.BUY_GET_GIFT=1]="BUY_GET_GIFT",n[n.SPEND_GET_GIFT=2]="SPEND_GET_GIFT",n),em=((i=em||{})[i.ORIGIN_PRICE=1]="ORIGIN_PRICE",i[i.DISCOUNT_PRICE=2]="DISCOUNT_PRICE",i),ep="_discounts_function_env",eg="_giveaway_gradient_gifts",ey="_sku_code_money",eh="_code_money",ev="_member_price",eb=["WS24","WSTD","WS7D","WSCP","WSPE","WSPD"];function e_(e,t){let r=e.filter(e=>e.variant?.id).map((e,t)=>{let r=e.variant,n=r.product,i=e.quantity||1,a=r.price?.amount?Number(r.price.amount):0,o=r.finalPrice?.amount===void 0?a:Number(r.finalPrice?.amount),u=new v.default(a).times(i).toNumber(),s=new v.default(o).times(i).toNumber();return{id:`temp-line-${t}-${r.id}`,name:n?.title||r.title||"",quantity:i,variantId:r.id,productId:n?.id||r.id.split("/").slice(0,-2).join("/"),amountPerQuantity:o,totalAmount:s,subtotalAmount:u,discountAllocations:[],customAttributes:e.attributes||[],variant:{id:r.id,price:o,listPrice:r.compareAtPrice?.amount?Number(r.compareAtPrice.amount):0,sku:r.sku||"",name:r.title||"",image:r.image?{url:r.image.url,altText:r.image.altText||void 0}:void 0,requiresShipping:r.requiresShipping??!0,availableForSale:r.availableForSale??!0,quantityAvailable:r.quantityAvailable??0,currentlyNotInStock:!1,weight:r.weight,metafields:r.metafields},product:n,path:n?.handle?`/products/${n.handle}`:"",discounts:[],options:r.selectedOptions?.map(e=>({name:e.name,value:e.value}))}}),n=r.reduce((e,t)=>e+t.subtotalAmount,0),i=r.reduce((e,t)=>e+t.totalAmount,0),a=e[0]?.variant?.price?.currencyCode;return{id:t?.id||"temp-cart-id",createdAt:t?.createdAt||new Date().toISOString(),currency:t?.currency||{code:a},taxesIncluded:t?.taxesIncluded,lineItems:r,totalLineItemsDiscount:0,orderDiscounts:0,lineItemsSubtotalPrice:n,subtotalPrice:n,totalPrice:i,totalTaxAmount:0,discountCodes:t?.discountCodes||[],discountAllocations:[],url:t?.url||"",ready:!0,customAttributes:t?.customAttributes,buyerIdentity:t?.buyerIdentity}}function eI(e,t,r){if(!e)return!0;let n=t?.getCartId(r);return!n||e===n}var eC=()=>{let e="u">typeof window?window.location.search:"",t={};if(-1!=e.indexOf("?")){let r=e.substr(1).split("&");for(let e=0;e<r.length;e++){let n=r[e]?.split("="),i=n?.[0],a=n?.[1];i&&a&&(t[i]=decodeURIComponent(a))}}return t},eA=(e,t,r)=>{let n=r?.all_store_variant??!1,i=e?.lineItems?.filter(e=>{let{is_gift:r}=ew(e.customAttributes);return n?!r:t?.find(t=>!r&&(0,g.atobID)(e.variantId)===t)});return i?.reduce((e,t)=>e+(r?.spend_money_type===1?Number(t.subtotalAmount)||0:Number(t.totalAmount)||0),0)||0},ew=(e=[])=>{let t=e.find(e=>e.key===ep);return(e=>{try{return JSON.parse(e)}catch(e){return{}}})(t?.value??"")??{}},e$=(e,t=[])=>{if(!t?.length)return!0;let r=new Set(e);return t.every(e=>r.has(e))},eP=(e,t=[])=>{if(!t?.length)return!0;let r=new Set(e);return!t.some(e=>r.has(e))};function ek(e,t,r){return!!Array.isArray(e)&&(0===e.length||e.some(e=>{let n=e$(t,e.with_user_tags)&&eP(t,e.without_user_tags),i=e$(r,e.with_special_url_params)&&eP(r,e.without_special_url_params);return n&&i}))}var eM=null,eS=null,eE=({qualifyingGift:e,giftProductsResult:t,locale:r})=>{let n=JSON.stringify({qualifyingGift:e?{spend:e.tier?.spend_sum_money,items:e.itemsToAdd?.map(e=>({variantId:e.variant.id,handle:e.variant.handle,sku:e.variant.sku,quantity:e.quantity??1,attributes:e.attributes}))||[]}:null,giftProductsLength:t?.length||0,giftProductsIds:t?.map(e=>e.id).sort()||[],locale:r});if(eS&&eS.key===n)return eS.result;let i=e?.itemsToAdd?.map((e,r)=>{let n=t?.find(t=>t.handle===e.variant.handle),i=n?.variants,a=Array.isArray(i)?i.find(t=>t.sku===e.variant.sku):void 0;return console.log("qualifyingGift variant",n,a),{line:{id:n?.id+"_"+r,variantId:String(a?.id),productId:String(n?.id),name:n?.name||n?.title||"",quantity:1,discounts:[],path:n?.handle||"",variant:a,totalAmount:0,subtotalAmount:new v.default("object"==typeof a?.price?a?.price?.amount||0:a?.price||0).toNumber(),options:[],discountAllocations:[],product:n,customAttributes:e.attributes,freeGiftVariant:void 0,relatedVariant:void 0},isSoldOut:!a?.availableForSale}})||[];return eS={key:n,result:i},i},eN=(e,t,r,n)=>{let i=(0,m.useMemo)(()=>r?.tags||[],[r?.tags]),a=(0,m.useMemo)(()=>!r,[r]),{client:o,locale:u}=ed(),s=(0,m.useRef)(null),l=(0,m.useMemo)(()=>n&&n.length>0?e_(n,e):e,[n,e]),{activeCampaign:d,subtotal:c}=(0,m.useMemo)(()=>{for(let e of t){let{rule_conditions:t=[],rule_result:r}=e,{spend_get_reward:n}=r||{};if(ek(t,i,[])&&n){let t=eA(l,n.main_product?.variant_list?.map(e=>e.variant_id)||[],{spend_money_type:n.main_product?.spend_money_type||1,variant_id_list:n.main_product?.variant_list?.map(e=>e.variant_id)||[],all_store_variant:n.main_product?.all_store_variant||!1});if(t>0)return{activeCampaign:e,subtotal:t}}}return{activeCampaign:null,subtotal:0}},[t,l,i,""]),{qualifyingTier:f,nextTierGoal:y,actualThreshold:h,currentCurrency:v}=(0,m.useMemo)(()=>{if(!d||!d.rule_result?.spend_get_reward?.gift_product)return{qualifyingTier:null,nextTierGoal:null,actualThreshold:0,currentCurrency:""};let e=d.rule_result.spend_get_reward.gift_product,t=l?.currency?.code||"";console.log("currentCurrency useCalcAutoFreeGift",l,t);let r=e=>e.spend_sum_money_multi_markets?.[t]?.value?Number(e.spend_sum_money_multi_markets[t].value):Number(e.spend_sum_money||0),n=[...e].sort((e,t)=>r(t)-r(e)).find(e=>c>=r(e)),i=e.find(e=>c<r(e)),a=n?r(n):0;return{qualifyingTier:n,nextTierGoal:i||null,actualThreshold:a,currentCurrency:t}},[d,c,l]),_=(0,m.useMemo)(()=>t.map(e=>e.rule_result?.spend_get_reward?.gift_product?.map(e=>e.reward_list.map(e=>e.variant_list.map(e=>e.handle))).flat()).flat().flat(2).filter(Boolean),[t]),I=(0,m.useMemo)(()=>!!_.length&&(!s.current||JSON.stringify(s.current.giftHandles)!==JSON.stringify(_)),[_]),{data:C}=(0,b.default)(I?_:null,async()=>{let e=await (0,p.getProductsByHandles)(o,{handles:_,locale:u}),t=Array.isArray(e)?e.filter(Boolean):[];return s.current={data:t,giftHandles:[..._]},t},{revalidateOnFocus:!1}),A=(0,m.useMemo)(()=>s.current&&!I?s.current.data||void 0:C,[C,I]);return{qualifyingGift:(0,m.useMemo)(()=>{if(!f||!d)return null;let e=f.reward_list?.map(e=>{if(!e.variant_list||0===e.variant_list.length)return null;let t=null;for(let r of e.variant_list){let e=A?.find(e=>e.handle===r.handle);if(e){let n=e.variants?.find(e=>e.sku===r.sku);if(n?.availableForSale){t=r;break}}}return t||(t=e.variant_list[0]),{variant:{id:(0,g.btoaID)(t.variant_id),handle:t.handle,sku:t.sku},quantity:e?.get_unit||1,attributes:[{key:ep,value:JSON.stringify({is_gift:!0,rule_id:d.rule_id,spend_sum_money:h,currency_code:v})}]}}).filter(e=>null!==e);return{tier:f,itemsToAdd:e}},[f,d,A,h,v]),nextTierGoal:y,activeCampaign:d,isLoading:a,giftProductsResult:A}},eT=({campaign:e,_giveaway:t,cart:r,locale:n,lines:i,profile:a})=>{let{client:o,locale:u}=ed(),s=n||u,[l,d]=(0,m.useState)(!1),c=(0,m.useRef)(null),f=(0,m.useMemo)(()=>i&&i.length>0?e_(i,r):r,[i,r]);(0,m.useEffect)(()=>{"au"===s&&d(!!y.default.get("points_subscribe"))},[s]);let g=(0,m.useMemo)(()=>{if(!e)return!1;let t=eC(),r=y.default.get("utm_campaign")||t?.utm_campaign;return(!e.activityAvailableQuery||!!r?.includes(e.activityAvailableQuery))&&(!e.requireLogin||!!a?.email)},[e,a]),[h,_]=(0,m.useMemo)(()=>{let e=1,t=0;return l&&(e=1.2,t=40),f?.lineItems?.forEach(({customAttributes:r})=>{r?.forEach(({key:r,value:n})=>{"_amount_upgrade_multiple"===r&&(e=Number(n)||1),"_amount_upgrade_value"===r&&(t=Number(n)||0)})}),[e,t]},[f?.lineItems,l]),I=(0,m.useMemo)(()=>g?(e?.breakpoints||[]).map(e=>({breakpoint:new v.default(e.breakpoint).minus(new v.default(_)).dividedBy(new v.default(h)).toFixed(2,v.default.ROUND_DOWN),giveawayProducts:e.giveawayProducts||[]})):[],[e,h,_]),C=(0,m.useMemo)(()=>[...new Set(I.flatMap(e=>e.giveawayProducts.map(e=>e.handle)).filter(Boolean))],[I]),A=(0,m.useMemo)(()=>!!C.length&&(!c.current||JSON.stringify(c.current.giftHandles)!==JSON.stringify(C)),[C]),w=(0,m.useMemo)(()=>g?(f?.lineItems||[]).filter(r=>{let n=r?.totalAmount&&Number(r.totalAmount)>0&&r.customAttributes?.every(e=>e.key!==t),i=r.product?.tags?.some(t=>e?.includeTags?.includes(t.trim())&&r.variant?.availableForSale);return n&&i}):[],[f?.lineItems,g,t]),$=(0,m.useMemo)(()=>g?w.reduce((t,r)=>{let n=e?.useTotalAmount?r.totalAmount:r.subtotalAmount;return new v.default(t).plus(new v.default(n||0))},new v.default(0)):new v.default(0),[w,g]),[P,k]=(0,m.useMemo)(()=>{if(!g)return[null,null];let e=[...I].sort((e,t)=>Number(t.breakpoint)-Number(e.breakpoint)),t=e.findIndex(e=>$.gte(new v.default(e.breakpoint))&&w.length>0);return -1===t?[null,e.length>0?e[e.length-1]??null:null]:[e[t]??null,t>0?e[t-1]??null:null]},[I,$,w.length]),{data:M}=(0,b.default)(A?C:null,async()=>{let e=await (0,p.getProductsByHandles)(o,{handles:C,locale:s}),t=Array.isArray(e)?e:[];return c.current={data:t,giftHandles:[...C]},t},{revalidateOnFocus:!1}),S=(0,m.useMemo)(()=>c.current&&!A?c.current.data||void 0:M,[M,A]);return{involvedLines:w,reorder:(0,m.useCallback)((r,n)=>{let i=r=>r.customAttributes?.some(e=>e.key===t)?0:r.product?.tags?.some(t=>e?.includeTags?.includes(t))?1:2;return i(n)-i(r)},[e?.includeTags,t]),disableCodeRemove:w.length>0,nextFreeGiftLevel:k,freeGiftLevel:P,involvedSubTotal:$,giftProductsResult:S}};function eq({updateCookie:e=!1,options:t}){let{client:r,locale:n,cartCookieAdapter:i}=ed(),{mutateCart:a,metafieldIdentifiers:o}=e8(),u=(0,m.useCallback)(async(t,{arg:n})=>{let u=await (0,p.createCart)(r,{...n,metafieldIdentifiers:o,cookieAdapter:i,updateCookie:e});return u&&u.discountCodes.filter(e=>!e.applicable).map(e=>e.code).length>0&&(u=await (0,p.updateCartCodes)(r,{cartId:u.id,discountCodes:u.discountCodes.filter(e=>e.applicable).map(e=>e.code),metafieldIdentifiers:o,cookieAdapter:i})),u&&a(u),u},[r,n,i,a,o]);return(0,_.default)("create-cart",u,t)}function eL(e){let{client:t,locale:r,cartCookieAdapter:n}=ed(),{mutateCart:i,metafieldIdentifiers:a}=e8(),o=(0,m.useCallback)(async(e,{arg:o})=>{let u,{cartId:s,lines:l,updateCookie:d=!0}=o,c=s||n?.getCartId(r);return(u=c?await (0,p.addCartLines)(t,{cartId:c,lines:l,metafieldIdentifiers:a,cookieAdapter:n}):await (0,p.createCart)(t,{lines:l,metafieldIdentifiers:a,cookieAdapter:n,updateCookie:d}))&&u.discountCodes.filter(e=>!e.applicable).map(e=>e.code).length>0&&(u=await (0,p.updateCartCodes)(t,{cartId:u.id,discountCodes:u.discountCodes.filter(e=>e.applicable).map(e=>e.code),metafieldIdentifiers:a,cookieAdapter:n})),u&&eI(o.cartId,n,r)&&i(u),u},[t,r,n,i,a]);return(0,_.default)("add-cart-lines",o,e)}var eD=e=>{if(!("u"<typeof window)&&(window.dataLayer=window?.dataLayer||[],Array.isArray(window.dataLayer)))try{window?.dataLayer?.push({event_parameters:null}),window?.dataLayer?.push(e||{})}catch(e){console.error("GA tracking error:",e)}};function eO(e=1e4){return new Promise((t,r)=>{let n=Date.now();!function i(){"u">typeof window&&void 0!==window.gtag?t():Date.now()-n>e?r(Error("GA4 gtag not loaded")):setTimeout(i,50)}()})}var eR=async(e="G-R0BRMRK4CY")=>{try{return await eO(),new Promise((t,r)=>{window.gtag?window.gtag("get",e,"client_id",e=>{e?t(e):r(Error("Failed to get client_id"))}):r(Error("gtag is not defined"))})}catch(e){return console.error("Failed to get GA4 client_id:",e),""}},ex=async(e="G-R0BRMRK4CY")=>{try{await eO();let t=new Promise(e=>{setTimeout(()=>{e({clientId:"",sessionId:""})},300)}),r=new Promise(t=>{window.gtag?window.gtag("get",e,"client_id",r=>{window.gtag("get",e,"session_id",e=>{t({clientId:r||"",sessionId:e||""})})}):t({clientId:"",sessionId:""})});return Promise.race([r,t])}catch(e){return console.error("Failed to get GA4 data:",e),{clientId:"",sessionId:""}}};function eU(e){let{client:t,locale:r,cartCookieAdapter:n}=ed(),{mutateCart:i,cart:a,metafieldIdentifiers:o}=e8(),u=(0,m.useCallback)(async(e,{arg:u})=>{let{cartId:s,discountCodes:l,replaceExistingCodes:d}=u;if(!l?.length)throw Error("Invalid input used for this operation: Miss discountCode");let c=await (0,p.updateCartCodes)(t,{cartId:s,discountCodes:d?l:[...l,...a?.discountCodes?.filter(e=>e.applicable).map(e=>e.code)||[]],cookieAdapter:n,metafieldIdentifiers:o}),f=l.filter(e=>c?.discountCodes?.find(t=>t.code===e&&!t.applicable));if(f.length)throw Error(`${f.join(", ")} is not applicable to the cart`);return c&&eI(u.cartId,n,r)&&i(c),c},[t,r,n,i,a,o]);return(0,_.default)("apply-codes",u,e)}function eH(e){let{client:t,locale:r,cartCookieAdapter:n}=ed(),{mutateCart:i,cart:a,metafieldIdentifiers:o}=e8(),u=(0,m.useCallback)(async(e,{arg:u})=>{let{cartId:s,discountCodes:l}=u,d=(a?.discountCodes?.filter(e=>!!e.applicable)||[]).filter(e=>l?.length?!l.includes(e.code):e.code).map(e=>e.code),c=await (0,p.updateCartCodes)(t,{cartId:s,discountCodes:d,metafieldIdentifiers:o,cookieAdapter:n});return c&&eI(u.cartId,n,r)&&i(c),c},[t,r,n,i,a,o]);return(0,_.default)("remove-codes",u,e)}function eF(e){let{client:t,locale:r,cartCookieAdapter:n}=ed(),{mutateCart:i,metafieldIdentifiers:a}=e8(),o=(0,m.useCallback)(async(e,{arg:o})=>{let u=await (0,p.updateCartLines)(t,{...o,metafieldIdentifiers:a,cookieAdapter:n});return u&&eI(o.cartId,n,r)&&i(u),console.log("use-update-cart-lines updatedCart",a,u),u},[t,r,n,i,a]);return(0,_.default)("update-cart-lines",o,e)}var eB=({cart:e,lineItems:t})=>t.map(t=>(({line:e})=>{let t=e.attributes||[],r=t.find(e=>e.key===ep),n=e.variant?.finalPrice?.amount??e.variant?.price?.amount;r||null==n||(t=t.concat([{key:ep,value:JSON.stringify({is_gift:!1,discounted_amount:new v.default(n).times(e.quantity||1).toNumber()})}]));let i=t.find(e=>e.key===ev),a=e.coupon;return!i&&a&&(t=t.concat([{key:ev,value:JSON.stringify({code:a.code})}])),!t.find(e=>e.key===ey||e.key===eh)&&a&&Number(a?.amount)>0&&(t=t.concat([{key:ey,value:new v.default(a.amount).times(e.quantity||1).toString()},{key:eh,value:new v.default(a.amount).times(e.quantity||1).toString()}])),{...e,attributes:t}})({line:(({cart:e,line:t})=>{let r=e?.lineItems.find(e=>e.variant.id===t.variant?.id),n=r?.customAttributes?.find(e=>e.key===ey),i=r?.customAttributes?.find(e=>e.key===eh),a=null;try{a=r?.customAttributes?.find(e=>e.key===ep&&JSON.parse(e.value)?.discounted_amount)}catch(e){}return n||a||i?{...t,attributes:[...t.attributes||[],n,a,i].filter(Boolean)}:t})({cart:e,line:t})}));function eG(e){let{client:t,locale:r,cartCookieAdapter:n}=ed(),{mutateCart:i,metafieldIdentifiers:a}=e8(),o=(0,m.useCallback)(async(e,{arg:o})=>{let{autoRemoveInvalidCodes:u=!0,onCodesRemoved:s,cartId:l,lineIds:d}=o,c=await (0,p.removeCartLines)(t,{cartId:l,lineIds:d,metafieldIdentifiers:a,cookieAdapter:n});if(c&&u){let e=c.discountCodes.filter(e=>!e.applicable).map(e=>e.code);if(e.length>0)if(s){let t=await s(c,e);t&&(c=t)}else c=await (0,p.updateCartCodes)(t,{cartId:c.id,discountCodes:c.discountCodes.filter(e=>e.applicable).map(e=>e.code),metafieldIdentifiers:a,cookieAdapter:n})||c}return c&&eI(o.cartId,n,r)&&i(c),c},[t,r,n,i,a]);return(0,_.default)("remove-cart-lines",o,e)}function ej(e){let t=e.customAttributes?.find(e=>"_discounts_function_env"===e.key)?.value;if(!t)return!1;try{let e=JSON.parse(t);return!!(e.is_gift&&e.rule_id&&e.spend_sum_money)}catch{return!1}}var eV=((a=eV||{})[a.PERCENTAGE=1]="PERCENTAGE",a[a.FIXED_AMOUNT=2]="FIXED_AMOUNT",a[a.REDUCE_PRICE=3]="REDUCE_PRICE",a),eW=((o=eW||{})[o.ORIGIN_PRICE=1]="ORIGIN_PRICE",o[o.MIN_DISCOUNTED_PRICE=2]="MIN_DISCOUNTED_PRICE",o);function eJ(e){if(!e?.clientId&&!e?.sessionId)return[];let t=[];return e.clientId&&t.push({key:"_ga4_client_id",value:e.clientId}),e.sessionId&&t.push({key:"_ga4_session_id",value:e.sessionId}),t}async function eK(){try{let e=await ex();return eJ(e)}catch(e){return console.error("Failed to get GA4 attributes:",e),[]}}function eQ({profile:e,customer:t,cart:r,memberType:n,currentUrl:i="",appContext:a,buyPath:o}){let u,s,l,d=(e=>{let t=y.default.get("userInfo");if(t){t=JSON.parse(t);let e="string"==typeof t?.id&&t?.id.split("/");t.setId=e[e.length-1]}if(!(t||e))return"new_user_unlogin";if(e){let{orders:t={}}=e;if(t?.edges?.length===1)return"old_user_orders_once";if(t?.edges?.length>1)return"old_user_orders_twice"}return"new_user_login"})(t),c=[{key:"_token",value:e?.token},{key:"_member_type",value:n??String(e?.memberType??0)},{key:"_user_type",value:d},{key:"_is_login",value:e?.token?"true":"false"}];e?.token&&c.push({key:"_login_user",value:"1"});let f=[{key:ep,value:JSON.stringify({user_tags:t?.tags||[]})}],m=[{key:"_presale",value:r?.lineItems?.some(e=>e?.variant?.metafields?.global?.presell==="presell")?"true":"false"}],p=[{key:"_hide_shipping",value:r?.lineItems?.some(e=>e?.variant?.metafields?.global?.hideShipping==="true")?"true":"false"}],h=[...c,...f,...m,{key:"_weight",value:r?.lineItems?.reduce((e,t)=>{let r=new v.default(t.variant.weight??0).times(t.quantity);return new v.default(e).plus(r).toNumber()},0).toString()},...p,{key:"utm_params",value:i},{key:"_app_source_name",value:a?.isInApp&&a?.appName?a.appName:"dtc"},{key:"_operating_system",value:(()=>{if("u"<typeof window||"u"<typeof navigator)return"Unknown";let e=navigator.userAgent||navigator.vendor||window.opera;return/iPad|iPhone|iPod/.test(e)&&!window.MSStream?"IOS":/android/i.test(e)?"Android":"Unknown"})()},{key:"_cart_id",value:r?.id},{key:"_buy_path",value:o},...(u=(0,g.getLocalStorage)("inviteCode")||y.default.get("inviteCode"),s=(0,g.getLocalStorage)("playModeId")||y.default.get("playModeId"),l=(0,g.getLocalStorage)("_popup")||y.default.get("_popup"),u&&s?l?[{key:"_invite_code",value:u||""},{key:"_play_mode_id",value:s||""},{key:"_popup",value:l}]:[{key:"_invite_code",value:u||""},{key:"_play_mode_id",value:s||""}]:[])].filter(e=>e?.value!==void 0&&e?.value!==null),b=r?.customAttributes?.filter(e=>!h.some(t=>t.key===e.key))||[];return[...h,...b].filter(e=>e?.value!==void 0&&e?.value!==null).map(e=>({key:e.key,value:"string"==typeof e.value?e.value:String(e.value)}))}var eY=({profile:e,customer:t,cart:r,memberType:n,appContext:i,buyPath:a})=>{let[o,u]=(0,m.useState)(""),[s,l]=(0,m.useState)(null);(0,m.useEffect)(()=>{u(window.location.href)},[]),(0,m.useEffect)(()=>{let e=!0;return ex().then(t=>{e&&l(t)}).catch(e=>{console.error("Failed to get GA4 data in useCartAttributes:",e)}),()=>{e=!1}},[]);let d=(0,m.useMemo)(()=>[...eQ({profile:e,customer:t,cart:r,memberType:n,currentUrl:o,appContext:i,buyPath:a}),...eJ(s)],[e,t,r,n,o,i,a,s]);return(0,m.useMemo)(()=>({attributes:d}),[d])},ez=((u=ez||{})[u.FREE=0]="FREE",u[u.MONTHLY=1]="MONTHLY",u[u.ANNUAL=2]="ANNUAL",u),eX=((s=eX||{}).MONTHLY="monthly",s.ANNUAL="annual",s),eZ=((l=eZ||{}).FREE="free",l.MONTHLY="monthly",l.ANNUAL="annual",l),e0=((d=e0||{}).FREE="free",d.TDD="tdd",d.NDD="ndd",d);function e1({memberSetting:e,cart:t}){return(0,m.useMemo)(()=>(function({memberSetting:e,cart:t}){let{plus_monthly_product:r,plus_annual_product:n}=e||{};if(!t?.lineItems)return{hasPlusMember:!1,hasMonthlyPlus:!1,hasAnnualPlus:!1};let i=t.lineItems.find(e=>e.product?.handle===r?.handle&&e.variant?.sku===r?.sku),a=t.lineItems.find(e=>e.product?.handle===n?.handle&&e.variant?.sku===n?.sku),o=!!i,u=!!a;return{hasPlusMember:o||u,hasMonthlyPlus:o,hasAnnualPlus:u,monthlyPlusItem:i,annualPlusItem:a}})({memberSetting:e,cart:t}),[e,t])}function e2({mutate:e,metafieldIdentifiers:t,disabled:r=!1,swrOptions:n}){let{client:i,locale:a,cartCookieAdapter:o}=ed(),u=(0,m.useCallback)(async(n,{arg:a})=>{if(r)return;let u=await (0,p.updateCartAttributes)(i,{...a,metafieldIdentifiers:t,cookieAdapter:o});return u&&e(u),u},[i,a,o,e,t,r]);return(0,_.default)("update-cart-attributes",u,n)}(0,m.createContext)({plusMemberMetafields:{},selectedPlusMemberMode:"free",setSelectedPlusMemberMode:()=>{},selectedShippingMethod:void 0,setSelectedShippingMethod:()=>{},showMoreShippingMethod:!1,setShowMoreShippingMethod:()=>{},variant:{},product:{},shippingMethodsContext:{freeShippingMethods:[],paymentShippingMethods:[],nddOverweight:!1,tddOverweight:!1,nddCoupon:void 0,tddCoupon:void 0,isLoadingCoupon:!1},selectedPlusMemberVariant:void 0,showPlusMemberBenefit:!1,setShowPlusMemberBenefit:()=>{},profile:void 0});var e9=((c=e9||{})[c.PERCENTAGE=1]="PERCENTAGE",c[c.FIXED_AMOUNT=2]="FIXED_AMOUNT",c),e5=((f=e5||{})[f.MIN_DISCOUNTED_PRICE=1]="MIN_DISCOUNTED_PRICE",f[f.MIN_TOTAL_PRICE=2]="MIN_TOTAL_PRICE",f);function e3(e){return e.variants.find(e=>e.availableForSale)||e.variants[0]}function e4({amount:e,currencyCode:t,locale:r,maximumFractionDigits:n,minimumFractionDigits:i,removeTrailingZeros:a}){let o=new Intl.NumberFormat(r,{style:"currency",currency:t,maximumFractionDigits:n??2,minimumFractionDigits:i??2}).format(e);return a&&(o=o.replace(/\.00$/,"")),o}var e6=(0,m.createContext)(null);function e7({options:e,onRemovingChange:t}){let{isRemoving:r}=function(e={}){let{removeFunctionGifts:t=!0,removeScriptGifts:r=!0,isGiftLineItem:n,runOnlyOnceAfterInit:i=!1,initDelay:a=500}=e,[o,u]=(0,m.useState)(!1),[s,l]=(0,m.useState)(!i),[d,c]=(0,m.useState)(!1),{cart:f}=e8(),{trigger:p}=eG(),g=(0,m.useMemo)(()=>f?.lineItems?f.lineItems.filter(e=>{if(t){let t=e.customAttributes?.find(e=>"_discounts_function_env"===e.key)?.value;if(t)try{let e=JSON.parse(t);if(e.is_gift&&e.rule_id&&e.spend_sum_money)return!0}catch(e){console.error("Failed to parse _discounts_function_env:",e)}}return!!(r&&e.customAttributes?.find(e=>"_giveaway_gradient_gifts"===e.key))||!!(n&&n(e))}):[],[f,t,r,n]);return S(()=>{!i||s||d||f?.lineItems?.length&&(l(!0),0===g.length&&c(!0))},[i,s,d,f?.lineItems,g.length],{trailing:!0,wait:a}),(0,m.useEffect)(()=>{i&&(!s||d)||o||0===g.length||(async()=>{u(!0);try{await p({lineIds:g.map(e=>e.id)})}catch(e){console.error("Failed to remove free gifts:",e)}finally{u(!1),i&&c(!0)}})()},[i,s,d,o,g,p]),{isRemoving:o}}(e);return(0,m.useEffect)(()=>{t(r)},[r,t]),null}function e8(e){let t=(0,m.useContext)(e6);if(!t&&!e?.optional)throw Error("useCartContext must be used within a CartProvider");return t}e.s(["BuyRuleType",0,ef,"CUSTOMER_ATTRIBUTE_KEY",0,ep,"CartProvider",0,function({children:e,functionAutoFreeGiftConfig:t,scriptAutoFreeGiftConfig:r,profile:n,customer:i,locale:a,metafieldIdentifiers:o,memberSetting:u,appContext:s,disableUpdateLineCodeAmountAttributes:l=!1}){let{client:d,cartCookieAdapter:c}=ed(),[f,g]=(0,m.useState)([]),[b,_]=(0,m.useState)([]),[I,C]=(0,m.useState)(!1),[A,w]=(0,m.useState)({editLineQuantityLoading:!1,editLineCodeAmountLoading:!1,listingAutoCodeApplying:!1,userCodeApplying:!1}),[$,P]=(0,m.useState)([]),[k,M]=(0,m.useState)([]),[S,E]=(0,m.useState)(!1),{run:N,data:T,loading:q,mutate:L}=ea(async()=>(0,p.getCart)(d,{cookieAdapter:c,metafieldIdentifiers:o}),{refreshDeps:[a]}),{trigger:D}=e2({mutate:L,metafieldIdentifiers:o,disabled:q}),{hasPlusMember:O}=e1({memberSetting:u,cart:T}),{attributes:R}=eY({profile:n,customer:i,cart:T,memberType:O?"2":String(n?.memberType??0),appContext:s});ea(()=>{let e,t=R.filter(e=>!f.some(t=>t.key===e.key)),r=b.filter(e=>!f.find(t=>t.key===e.key)),n=[...t,...f],i=T&&(e=T.customAttributes,n.some(t=>!e.some(e=>e.key===t.key))||e.some(e=>{let t=n.find(t=>t.key===e.key);return!t||t.value!==e.value})||r.some(t=>e.some(e=>e.key===t.key))),a=n.filter(e=>!r.find(t=>t.key===e.key));return i?D({attributes:a}):Promise.resolve(T)},{throttleWait:1e3,throttleTrailing:!0,throttleLeading:!1,refreshDeps:[R,f,b]}),(({cart:e,mutateCart:t,isCartLoading:r,setLoadingState:n,metafieldIdentifiers:i,disabled:a=!1})=>{let{client:o,cartCookieAdapter:u}=ed(),s=(0,m.useMemo)(()=>e?.discountCodes.filter(({code:e,applicable:t})=>t&&eb.some(t=>e.startsWith(t))).map(({code:e})=>e),[e]),l=(0,m.useMemo)(()=>e?.lineItems.map(e=>{let t=[],r=[],n=e.discountAllocations?.find(e=>s?.includes(e.code)),i=ew(e.customAttributes);Number(i.discounted_amount)===Number(e.totalAmount)||i.is_gift||t.push({key:ep,value:JSON.stringify({...i,discounted_amount:Number(e.totalAmount)})});let a=n?.amount||0,o=e.customAttributes?.find(e=>e.key===ey||e.key===eh),u=e.customAttributes?.find(e=>e.key===ey&&e.value===String(a))&&e.customAttributes?.find(e=>e.key===eh&&e.value===String(a));return n&&!u?(t.push({key:ey,value:String(a)}),t.push({key:eh,value:String(a)})):!n&&o&&(r.push(ey),r.push(eh)),{line:e,attrNeedUpdate:t,attrNeedDelete:r}}).filter(({attrNeedUpdate:e,attrNeedDelete:t})=>e.length||t.length).map(({line:e,attrNeedUpdate:t,attrNeedDelete:r})=>{let n=e.id,i=e.customAttributes||[];return r.length&&(i=i.filter(e=>!r.includes(e.key))),t.length&&(i=i.filter(e=>!t.some(t=>t.key===e.key)).concat(t)),{id:n,attributes:i}}),[e?.lineItems,s]),{loading:d}=ea(async()=>{if(l?.length&&!r&&!a){let r=await (0,p.updateCartLines)(o,{cartId:e?.id||"",lines:l,metafieldIdentifiers:i,cookieAdapter:u});r&&t(r)}},{throttleWait:3e3,throttleTrailing:!0,refreshDeps:[l,r,a]});(0,m.useEffect)(()=>{n(e=>({...e,editLineCodeAmountLoading:d}))},[d,n])})({cart:T,mutateCart:L,isCartLoading:q||I,setLoadingState:w,metafieldIdentifiers:o,disabled:l});let x=(0,m.useCallback)(e=>{_(e)},[_]),U=(0,m.useCallback)(e=>{g(t=>[...t.filter(t=>!e.some(e=>e.key===t.key)),...e])},[g]),H=eN(T,t||[],i),F=eT({campaign:r,_giveaway:eg,cart:T,profile:n}),B=(0,m.useMemo)(()=>(({scriptAutoFreeGiftResult:e,gradient_gifts:t,locale:r})=>{let n=JSON.stringify({freeGiftLevel:e?.freeGiftLevel?{items:e.freeGiftLevel.giveawayProducts?.map(e=>({handle:e.handle,sku:e.sku,quantity:1}))||[]}:null,giftProductsLength:e?.giftProductsResult?.length||0,giftProductsIds:e?.giftProductsResult?.map(e=>e.id).sort()||[],gradientGiftsId:t?.id||t?.name||"",locale:r});if(eM&&eM.key===n)return eM.result;let i=e?.freeGiftLevel?.giveawayProducts?.filter(t=>e?.giftProductsResult?.some(e=>e.handle===t.handle)).map((n,i)=>{let a=e?.giftProductsResult?.find(e=>e.handle===n.handle),o=a?.variants,u=Array.isArray(o)?o.find(e=>e.sku===n.sku):void 0,s=eC(),l=y.default.get("utm_campaign")||s?.utm_campaign,d=t.activityAvailableQuery&&l?.includes(t.activityAvailableQuery),c=!1;"au"===r&&(c=!!y.default.get("points_subscribe"));let f=[{key:"_giveaway_gradient_gifts",value:"_giveaway_gradient_gifts"},...c?[{key:"_amount_upgrade_multiple",value:"1.2"},{key:"_amount_upgrade_value",value:"40"}]:[],...d&&t?.activityQroperty?[{key:t.activityQroperty,value:t.activityQroperty}]:[]];return{line:{id:a?.id+"_"+i,variantId:String(u?.id),productId:String(a?.id),name:a?.name||a?.title||"",quantity:1,discounts:[],path:a?.handle||"",variant:u,totalAmount:0,subtotalAmount:new v.default("object"==typeof u?.price?u?.price?.amount||0:u?.price||0).toNumber(),options:[],discountAllocations:[],product:a,customAttributes:f,freeGiftVariant:void 0,relatedVariant:void 0},isSoldOut:!u?.availableForSale}})||[];return eM={key:n,result:i},i})({scriptAutoFreeGiftResult:F,gradient_gifts:r,locale:a}),[F,r,a]),G=(0,m.useMemo)(()=>eE({qualifyingGift:H?.qualifyingGift||null,giftProductsResult:H?.giftProductsResult||[],locale:a}),[H,a]);(0,m.useEffect)(()=>{A.userCodeApplying||A.editLineQuantityLoading||A.editLineCodeAmountLoading||A.listingAutoCodeApplying||(P(B),M(G))},[B,G,A.userCodeApplying,A.editLineQuantityLoading,A.editLineCodeAmountLoading,A.listingAutoCodeApplying]);let j=(0,m.useMemo)(()=>{let e=T?.lineItems.filter(e=>e.customAttributes?.some(e=>[ep,eg].includes(e.key)));return[...$,...k].filter(t=>!e?.find(e=>e.variantId===t.line.variantId&&e.customAttributes?.some(t=>"_discounts_function_env"===t.key&&ew(e.customAttributes).is_gift)))},[T?.lineItems,$,k]),V=(0,m.useMemo)(()=>(T?.lineItems.reduce((e,t)=>e+t.quantity,0)||0)+(j?.reduce((e,t)=>e+(t.quantity||1),0)||0),[T?.lineItems,j]),W=(0,m.useMemo)(()=>({removeFunctionGifts:!!t,removeScriptGifts:!!r,runOnlyOnceAfterInit:!0,initDelay:500}),[t,r]),J=(0,m.useMemo)(()=>({totalQuantity:V,cart:T,isCartLoading:q,triggerFetch:N,mutateCart:L,addCustomAttributes:U,removeCustomAttributes:x,locale:a,profile:n,customer:i,isCodeChanging:I,setIsCodeChanging:C,functionAutoFreeGiftConfig:t,scriptAutoFreeGiftConfig:r,setLoadingState:w,loadingState:A,functionAutoFreeGift:k,functionAutoFreeGiftResult:H,setFunctionAutoFreeGift:M,scriptAutoFreeGift:$,scriptAutoFreeGiftResult:F,setScriptAutoFreeGift:P,giftNeedAddToCartLines:j,metafieldIdentifiers:o,memberSetting:u,appContext:s,autoRemoveFreeGiftsOptions:W,isAutoRemovingFreeGifts:S}),[T,V,q,N,L,U,x,a,I,t,r,A,k,H,M,$,F,P,j,o,i,n,u,s,W,S]);return(0,h.jsxs)(e6.Provider,{value:J,children:[(t||r)&&(0,h.jsx)(e7,{options:W,onRemovingChange:E}),e]})},"ShopifyContext",0,eo,"ShopifyProvider",0,function({config:e,locale:t,locales:r,cookieAdapter:n=eu,cartCookieAdapter:i=el,routerAdapter:a,userAdapter:o,children:u,performanceAdapter:s}){let l=(0,m.useMemo)(()=>(0,p.createShopifyClient)(e,t),[e,t]),d=(0,m.useMemo)(()=>({client:l,config:e,locale:t,locales:r||e.getLocales(),cookieAdapter:n,cartCookieAdapter:i,routerAdapter:a,userAdapter:o,performanceAdapter:s}),[l,e,t,r,n,i,a,s,o]);return(0,h.jsx)(eo.Provider,{value:d,children:u})},"formatFunctionAutoFreeGift",0,eE,"gaTrack",0,eD,"getDiscountEnvAttributeValue",0,ew,"getGA4ClientId",0,eR,"getMatchedMainProductSubTotal",0,eA,"isAnyGift",0,function(e){return ej(e)||(e.customAttributes?.some(e=>"_giveaway_gradient_gifts"===e.key)??!1)||(e.customAttributes?.some(e=>"_freegift_related_handlesku"===e.key)??!1)},"isFunctionGift",0,ej,"preCheck",0,ek,"useAddCartLines",0,eL,"useAddToCart",0,function({withTrack:e=!0}={},t){let{client:r,config:n,locale:i,cartCookieAdapter:a,userAdapter:o,performanceAdapter:u}=ed(),{cart:s,addCustomAttributes:l,memberSetting:d,profile:c,customer:f}=e8(),{trigger:p}=eU(),{trigger:y}=eH(),{trigger:h}=eL(),{trigger:b}=eF(),{trigger:I}=eq({updateCookie:!0}),{hasPlusMember:C}=e1({memberSetting:d,cart:s}),{attributes:A}=eY({profile:c,customer:f,cart:s,memberType:C?"2":String(c?.memberType??0),buyPath:"hasCart"}),w=(0,m.useCallback)(async(t,{arg:r})=>{let{lineItems:i,cartId:a,discountCodes:o,gtmParams:d={},buyerIdentity:c,needCreateCart:f=!1,onCodesInvalid:m,replaceExistingCodes:_,customAttributes:C}=r;if(!i||0===i.length)return;u?.addToCartStart();let w=eB({cart:s,lineItems:i}).map(e=>({merchandiseId:e.variant?.id||"",quantity:e.quantity||1,attributes:e.attributes,sellingPlanId:e.sellingPlanId})).filter(e=>e.merchandiseId&&e.quantity);if(0===w.length)return;let $=f?void 0:a||s?.id,P=null;if($){let e=[],t=w.filter(t=>{let r=s?.lineItems?.find(e=>e.variant?.id===t.merchandiseId);return!r||(e.push({id:r.id,quantity:r.quantity+(t.quantity||1),attributes:t.attributes||void 0}),!1)});if(e.length>0&&(P=await b({cartId:$,lines:e})),t.length>0&&(P=await h({cartId:P?.id||$,lines:t})),P||(P=s),console.log("npm addCartLines resultCart",P),P&&P.discountCodes&&P.discountCodes.length>0){let e=P.discountCodes.filter(e=>!e.applicable).map(e=>e.code);if(e.length>0)if(m){let t=await m(P,e);t&&(P=t)}else await y({discountCodes:e})}P&&o&&o.length>0&&p({replaceExistingCodes:_,discountCodes:o}),C&&C.length>0&&l(C)}else P=await I({lines:w,buyerIdentity:c,discountCodes:o,customAttributes:[...A,...C||[]]});if(e&&P){let e=i.map(e=>{let t=P.lineItems.find(t=>t.variant.id===e.variant?.id);return t?{...t,finalPrice:e.variant?.finalPrice,quantity:e.quantity,gtmParams:e.gtmParams}:null}).filter(Boolean);(({lineItems:e=[],gtmParams:t={}})=>{if(!e.length)return;let r=e[0].product?.price?.currencyCode,n=e.reduce((e,t)=>e.plus(t.finalPrice?.amount||t.totalAmount||0),new v.default(0)).toNumber();eD({event:"ga4Event",event_name:"add_to_cart",event_parameters:{page_group:t?.pageGroup,currency:r,value:n,position:t?.position||"",items:e.map(e=>{let r=e.variant.image?.url||e.product?.images?.[0]?.url,n=e.gtmParams?.item_category_id,i=e.variant.id?(0,g.atobID)(e.variant.id):void 0;return{item_id:e.variant.sku,item_name:e.product?.title,item_brand:t?.brand||"",item_category:e.product?.productType||"",item_variant:e.variant.name,price:e.variant.listPrice??e.variant.price,quantity:e.quantity||1,...r&&{image_url:r},...void 0!==n&&{item_category_id:n},...i&&{item_variant_id:i}}}),...t?.ga4Params}})})({lineItems:e,gtmParams:{...d,brand:n.getBrand()}}),(({lineItems:e=[]})=>{if("u">typeof window&&window.fbq&&e.length){let{variant:t,quantity:r,product:n}=e[0];try{window.fbq("track","AddToCart",{value:t.listPrice||t.price,num_items:r,currency:n?.price?.currencyCode,content_name:n?.title,content_type:"product_group",content_ids:String(t.id),content_category:n?.metafields?.global?.trafficType||"public"})}catch(e){console.error("FBQ tracking error:",e)}}})({lineItems:e})}return u?.addToCartEnd(),P},[r,i,a,o,s,e,u,I,h,b,p,y,l,n,A]);return(0,_.default)("add-to-cart",w,t)},"useApplyCartCodes",0,eU,"useBuyNow",0,function({withTrack:e=!0}={},t){let{client:r,config:n,locale:i,cartCookieAdapter:a,userAdapter:o}=ed(),{profile:u,customer:s,memberSetting:l,appContext:d}=e8(),c=o?.isLoggedIn||!1,f=(0,m.useCallback)(async(t,{arg:i})=>{let{lineItems:o,discountCodes:c,gtmParams:f={},buyerIdentity:m,fbqTrackConfig:y,customAttributes:h,metafieldIdentifiers:b,redirectToCheckout:_=!0}=i;if(!o||0===o.length)return;let{hasPlusMember:I}=function({memberSetting:e,lines:t}){let{plus_monthly_product:r,plus_annual_product:n}=e||{};if(!t||0===t.length)return{hasPlusMember:!1,hasMonthlyPlus:!1,hasAnnualPlus:!1};let i=t.find(e=>{let t=e.variant?.product?.handle,n=e.variant?.sku;return t===r?.handle&&n===r?.sku}),a=t.find(e=>{let t=e.variant?.product?.handle,r=e.variant?.sku;return t===n?.handle&&r===n?.sku}),o=!!i,u=!!a;return{hasPlusMember:o||u,hasMonthlyPlus:o,hasAnnualPlus:u,monthlyPlusLine:i,annualPlusLine:a}}({memberSetting:l,lines:o}),C=I?"2":String(u?.memberType??0),A=eB({lineItems:o}).map(e=>({merchandiseId:e.variant?.id||"",quantity:e.quantity||1,attributes:e.attributes,sellingPlanId:e.sellingPlanId})).filter(e=>e.merchandiseId);if(0===A.length)return;let w=eQ({profile:u,customer:s,memberType:C,cart:{lineItems:o},currentUrl:window.location.href,appContext:d,buyPath:"buyNow"}),$=await eK(),P=await (0,p.createCart)(r,{lines:A,metafieldIdentifiers:b,cookieAdapter:a,buyerIdentity:m,discountCodes:c,customAttributes:[...w.filter(e=>!(h||[]).some(t=>t.key===e.key)),...$.filter(e=>!(h||[]).some(t=>t.key===e.key)),...h||[]]});if(!P)throw Error("Failed to create cart for buy now");if(e&&P.lineItems&&((({lineItems:e=[],gtmParams:t={}})=>{if(!e.length)return;let r=e[0].product?.price?.currencyCode,n=e.reduce((e,t)=>e.plus(t.totalAmount||0),new v.default(0)).toNumber();eD({event:"ga4Event",event_name:"begin_checkout",event_parameters:{page_group:t?.pageGroup,position:t?.position||"",currency:r,value:n,items:e.map(e=>{let r=e.variant.image?.url||e.product?.images?.[0]?.url,n=e.gtmParams?.item_category_id,i=e.variant.id?(0,g.atobID)(e.variant.id):void 0;return{item_id:e.variant.sku,item_name:e.product?.title,item_brand:t?.brand||"",item_category:e.product?.productType||"",item_variant:e.variant.name,price:e.variant.listPrice??e.variant.price,quantity:e.quantity||1,...r&&{image_url:r},...void 0!==n&&{item_category_id:n},...i&&{item_variant_id:i}}}),...t?.ga4Params}})})({lineItems:o.map(e=>{let t=P.lineItems.find(t=>t.variant.id===e.variant?.id);return t?{...t,quantity:e.quantity}:null}).filter(Boolean),gtmParams:{...f,brand:n.getBrand()}}),y&&(({trackConfig:e})=>{if("u">typeof window)try{e?.fbqBuyNowEvent&&window.fbq&&window.fbq("trackCustom",e.fbqBuyNowEvent),e?.gtagBuyNowLabel&&e?.gtagId&&window.gtag&&window.gtag("event",e.gtagBuyNowConversion||"conversion",{send_to:`${e.gtagId}/${e.gtagBuyNowLabel}`})}catch(e){console.error("Buy Now tracking error:",e)}})({trackConfig:y})),_)if(P.url)"u">typeof window&&(window.location.href=P.url);else throw Error("Failed to get checkout URL");return P},[r,i,c,a,e,s,u,l,d]);return(0,_.default)("buy-now",f,t)},"useCalcGiftsFromLines",0,function({lines:e,customer:t,scriptGiveawayKey:r=eg}){let{locale:n}=ed(),{cart:i,functionAutoFreeGiftConfig:a,scriptAutoFreeGiftConfig:o}=e8(),u=eN(i,a||[],t,e),s=eT({campaign:o,_giveaway:r,cart:i,locale:n,lines:e}),l=(0,m.useMemo)(()=>[...(u.qualifyingGift?.itemsToAdd||[]).map(e=>{let t=u.giftProductsResult?.find(t=>t.handle===e.variant.handle),r=t?.variants,n=Array.isArray(r)?r.find(t=>t.sku===e.variant.sku):void 0;return n?{variant:{...n,product:t},quantity:e.quantity??1,attributes:e.attributes}:(console.warn(`Function gift: Variant not found for handle=${e.variant.handle}, sku=${e.variant.sku}`),null)}).filter(e=>null!==e),...s.freeGiftLevel?s.freeGiftLevel.giveawayProducts.map(e=>{let t=s.giftProductsResult?.find(t=>t.handle===e.handle),n=t?.variants,i=Array.isArray(n)?n.find(t=>t.sku===e.sku):void 0;return i?{variant:{...i,product:t},quantity:1,attributes:[{key:r,value:r}]}:(console.warn(`Script gift: Variant not found for handle=${e.handle}, sku=${e.sku}`),null)}).filter(e=>null!==e):[]],[u.qualifyingGift,u.giftProductsResult,s.freeGiftLevel,s.giftProductsResult,r]),d=(0,m.useMemo)(()=>l.length>0,[l]);return{functionGift:u,scriptGift:s,allGiftLines:l,hasGifts:d}},"useCalcOrderDiscount",0,(e,t,r)=>{let n=(0,m.useMemo)(()=>r?.tags||[],[r?.tags]),i=(0,m.useMemo)(()=>!r,[r]),{activeCampaign:a,subtotal:o}=(0,m.useMemo)(()=>{for(let r of t||[]){let{rule_conditions:t=[],result_detail:i}=r,{main_product:a,order_discount_conf:o}=i||{};if(ek(t,n,[])&&a&&o){let t=eA(e,a?.variant_list?.map(e=>e.variant_id)||[],{spend_money_type:2===o.base_price?2:1,variant_id_list:a?.variant_list?.map(e=>e.variant_id)||[],all_store_variant:a?.all_store_variant||!1});if(t>0)return{activeCampaign:r,subtotal:t}}}return{activeCampaign:null,subtotal:0}},[t,e,n,""]),{qualifyingDiscount:u,nextTierGoal:s,discountAmount:l}=(0,m.useMemo)(()=>{if(!a||!a.result_detail?.order_discount_conf?.tiered_discounts)return{qualifyingDiscount:null,nextTierGoal:null,discountAmount:0};let t=e?.currency?.code||"";console.log("currentCurrency",e,t);let r=a.result_detail.order_discount_conf,n=r.tiered_discounts_markets?.[t]||r.tiered_discounts,i=[...n].sort((e,t)=>Number(t.amount)-Number(e.amount)).find(e=>o>=Number(e.amount)),u=[...n].sort((e,t)=>Number(e.amount)-Number(t.amount)).find(e=>o<Number(e.amount));if(!i)return{qualifyingDiscount:null,nextTierGoal:u||null,discountAmount:0};let s=0;switch(i.discount_type){case 1:s=o*i.discount/100;break;case 2:s=i.discount;break;case 3:s=Math.min(o,i.discount);break;default:s=0}return{qualifyingDiscount:i,nextTierGoal:u||null,discountAmount:s}},[a,o]);return{qualifyingDiscount:u,nextTierGoal:s,activeCampaign:a,discountAmount:l,cartTotalForDiscount:o,isLoading:i}},"useCartContext",0,e8,"useCreateCart",0,eq,"usePrice",0,function({amount:e,baseAmount:t,currencyCode:r,soldOutDescription:n="",maximumFractionDigits:i,minimumFractionDigits:a,removeTrailingZeros:o}){let{locale:u}=ed(),s=(0,m.useMemo)(()=>"number"==typeof e&&r?n&&e>=9999999.99?n:t?function({amount:e,baseAmount:t,currencyCode:r,locale:n,maximumFractionDigits:i,minimumFractionDigits:a,removeTrailingZeros:o}){return{price:e4({amount:e,currencyCode:r,locale:n,maximumFractionDigits:i,minimumFractionDigits:a,removeTrailingZeros:o}),basePrice:e4({amount:t,currencyCode:r,locale:n,maximumFractionDigits:i,minimumFractionDigits:a,removeTrailingZeros:o})}}({amount:e,baseAmount:t,currencyCode:r,locale:u,maximumFractionDigits:i,minimumFractionDigits:a,removeTrailingZeros:o}):e4({amount:e,currencyCode:r,locale:u,maximumFractionDigits:i,minimumFractionDigits:a,removeTrailingZeros:o}):"",[e,t,r,u,i,a,n,o]);return(0,m.useMemo)(()=>{let t=!!(e&&e<=0);return"string"==typeof s?{price:s,basePrice:s,free:t}:{...s,free:t}},[s,e])},"useProduct",0,function(e={}){let{client:t,locale:r}=ed(),{handle:n,metafieldIdentifiers:i,...a}=e;return(0,b.default)(n?["product",r,n,i]:null,()=>(0,p.getProduct)(t,{handle:n,locale:r,metafieldIdentifiers:i}),a)},"useProductsByHandles",0,function(e={}){let{client:t,locale:r}=ed(),{handles:n,metafieldIdentifiers:i,...a}=e,o=new Set(n||[]),u=o?[...o].sort():void 0;return(0,b.default)(u&&u.length>0?["products-by-handles",r,u.join(","),i]:null,()=>{let e=[...o];if(e?.length===0)throw Error("Handles are required");return(0,p.getProductsByHandles)(t,{handles:[...o],locale:r,metafieldIdentifiers:i})},{revalidateOnFocus:!1,...a})},"useRemoveCartCodes",0,eH,"useRemoveCartLines",0,eG,"useSelectedOptions",0,function(e,t){let[r,n]=(0,m.useState)({});return(0,m.useEffect)(()=>{if(!e||!e.variants.length)return void n({});let r=e.variants[0];if("u">typeof window){let n=new URLSearchParams(window.location.search).get("variant");if(n){let i=e.variants.find(e=>t?e.sku===t:e.id===n||e.id.includes(n)||function(e){try{let t=e.split("/").pop()||"";return atob(t)}catch{return e}}(e.id)===n);i&&(r=i)}}r&&n(r.selectedOptions.reduce((e,t)=>(e[t.name]=t.value,e),{}))},[e,t]),[r,n]},"useShopify",0,ed,"useUpdateBuyerIdentity",0,function({mutate:e,metafieldIdentifiers:t,disabled:r=!1,swrOptions:n}){let{client:i,locale:a,cartCookieAdapter:o}=ed(),u=(0,m.useCallback)(async(n,{arg:a})=>{if(r)return;let u=await (0,p.updateBuyerIdentity)(i,{...a,metafieldIdentifiers:t,cookieAdapter:o});return u&&e(u),u},[i,a,o,e,t,r]);return(0,_.default)("update-buyer-identity",u,n)},"useUpdateCartAttributes",0,e2,"useUpdateCartLines",0,eF,"useVariant",0,function({product:e,selectedOptions:t}){let[r,n]=(0,m.useState)(e?e3(e):void 0);return(0,m.useEffect)(()=>{if(!e)return void n(void 0);let i=e.variants.find(e=>e.selectedOptions.every(e=>t[e.name]===e.value));i&&i.id!==r?.id?n(i):i||n(e3(e))},[t,e,r?.id]),r}],141376)}]);