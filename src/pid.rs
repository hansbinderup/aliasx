use std::env;

pub const PID_ALIAS : &str = "ALIASX_PID";
const PID_PATH : &str = "/tmp/aliasx/";

pub fn is_pid_set() -> bool {
    return env::var(PID_ALIAS).is_ok();
}

pub fn try_get_file() -> Option<String> {
    env::var(PID_ALIAS).ok().map(|pid| format!("{}{}", PID_PATH, pid))
}

