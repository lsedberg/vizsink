#[macro_export]
macro_rules! console_log {
    ( $( $arg:expr ),* $(,)? ) => {
        {
            let arr = web_sys::js_sys::Array::new();
            $(
                arr.push(&($arg).into());
            )*
            web_sys::console::log(&arr);
        }
    }
}

#[macro_export]
macro_rules! console_info {
    ( $( $arg:expr ),* $(,)? ) => {
        {
            let arr = web_sys::js_sys::Array::new();
            $(
                arr.push(&($arg).into());
            )*
            web_sys::console::info(&arr);
        }
    }
}

#[macro_export]
macro_rules! console_error {
    ( $( $arg:expr ),* $(,)? ) => {
        {
            let arr = web_sys::js_sys::Array::new();
            $(
                arr.push(&($arg).into());
            )*
            web_sys::console::error(&arr);
        }
    }
}

#[macro_export]
macro_rules! console_warn {
    ( $( $arg:expr ),* $(,)? ) => {
        {
            let arr = web_sys::js_sys::Array::new();
            $(
                arr.push(&($arg).into());
            )*
            web_sys::console::warn(&arr);
        }
    }
}
