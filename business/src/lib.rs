pub mod application {
    pub mod product {
        pub mod create;
        pub mod delete;
        pub mod estimate_expiry;
        pub mod get_all;
        pub mod get_by_id;
        pub mod identify;
        pub mod scan_receipt;
        pub mod update;
    }
    pub mod suggestion {
        pub mod generate;
    }
}

pub mod domain {
    pub mod errors;
    pub mod logger;
    pub mod product {
        pub mod errors;
        pub mod model;
        pub mod repository;
        pub mod services;
        pub mod urgency;
        pub mod value_objects;
        pub mod use_cases {
            pub mod create;
            pub mod delete;
            pub mod estimate_expiry;
            pub mod get_all;
            pub mod get_by_id;
            pub mod identify;
            pub mod scan_receipt;
            pub mod update;
        }
    }
    pub mod suggestion {
        pub mod errors;
        pub mod model;
        pub mod services;
        pub mod use_cases {
            pub mod generate;
        }
    }
}
