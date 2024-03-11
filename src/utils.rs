pub fn init_logger() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use simple_logger::SimpleLogger;
        use log::LevelFilter::Debug;
        SimpleLogger::new().with_colors(true).with_level(Debug).init().unwrap();
    }
    #[cfg(target_arch = "wasm32")]
    {
        use console_log::init;
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
    }
    log::info!("Enabled logging");
}
