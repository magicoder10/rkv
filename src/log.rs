#[macro_export]
macro_rules! debug {
    ($logger:expr, $fmt:expr) => {
        slog::debug!($logger, $fmt; "file" => file!(), "line" => line!())
    };
    ($logger:expr, $fmt:expr, $($arg:tt)*) => {
        slog::debug!($logger, $fmt, $($arg)*; "file" => file!(), "line" => line!())
    };
    ($logger:expr, $fmt:expr, $($arg:tt)*; $($key:expr, $value:expr), +) => {
         slog::debug!($logger, $fmt, $($arg)*; "file" => file!(), "line" => line!(), $($key => $value), +)
    };
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $fmt:expr) => {
        slog::info!($logger, $fmt; "file" => file!(), "line" => line!())
    };
    ($logger:expr, $fmt:expr, $($arg:tt)*) => {
        slog::info!($logger, $fmt, $($arg)*; "file" => file!(), "line" => line!())
    };
    ($logger:expr, $fmt:expr, $($arg:tt)*; $($key:expr, $value:expr), +) => {
         slog::info!($logger, $fmt, $($arg)*; "file" => file!(), "line" => line!(), $($key => $value), +)
    };
}

#[macro_export]
macro_rules! error {
    ($logger:expr, $fmt:expr) => {
        slog::error!($logger, $fmt; "file" => file!(), "line" => line!())
    };
    ($logger:expr, $fmt:expr, $($arg:tt)*) => {
        slog::error!($logger, $fmt, $($arg)*; "file" => file!(), "line" => line!())
    };
    ($logger:expr, $fmt:expr, $($arg:tt)*; $($key:expr, $value:expr), +) => {
        slog::error!($logger, $fmt, $($arg)*; "file" => file!(), "line" => line!(), $($key => $value), +)
    };
}