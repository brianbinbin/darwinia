// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Custom panic hook with bug report link
//!
//! This crate provides the [`set`] function, which wraps around [`std::panic::set_hook`] and
//! sets up a panic hook that prints a backtrace and invites the user to open an issue to the
//! given URL.
//!
//! By default, the panic handler aborts the process by calling [`std::process::exit`]. This can
//! temporarily be disabled by using an [`AbortGuard`].
use backtrace::Backtrace;
use std::io::{self, Write};
use std::marker::PhantomData;
use std::panic::{self, PanicInfo};
use std::cell::Cell;
use std::thread;
use std::env;

thread_local! {
	static ABORT: Cell<bool> = Cell::new(true);
}

/// Set the panic hook.
///
/// Calls [`std::panic::set_hook`] to set up the panic hook.
///
/// The `bug_url` parameter is an invitation for users to visit that URL to submit a bug report
/// in the case where a panic happens.
pub fn set(bug_url: &'static str) {
    panic::set_hook(Box::new(move |c| panic_hook(c, bug_url)));
}

macro_rules! ABOUT_PANIC {
	() => ("
This is a bug. Please report it at:

	{}
")}

/// Set aborting flag. Returns previous value of the flag.
fn set_abort(enabled: bool) -> bool {
    ABORT.with(|flag| {
        let prev = flag.get();
        flag.set(enabled);
        prev
    })
}

/// RAII guard for whether panics in the current thread should unwind or abort.
///
/// Sets a thread-local abort flag on construction and reverts to the previous setting when dropped.
/// Does not implement `Send` on purpose.
///
/// > **Note**: Because we restore the previous value when dropped, you are encouraged to leave
/// > the `AbortGuard` on the stack and let it destroy itself naturally.
pub struct AbortGuard {
    /// Value that was in `ABORT` before we created this guard.
    previous_val: bool,
    /// Marker so that `AbortGuard` doesn't implement `Send`.
    _not_send: PhantomData<std::rc::Rc<()>>,
}

impl AbortGuard {
    /// Create a new guard. While the guard is alive, panics that happen in the current thread will
    /// unwind the stack (unless another guard is created afterwards).
    pub fn force_unwind() -> AbortGuard {
        AbortGuard {
            previous_val: set_abort(false),
            _not_send: PhantomData,
        }
    }

    /// Create a new guard. While the guard is alive, panics that happen in the current thread will
    /// abort the process (unless another guard is created afterwards).
    pub fn force_abort() -> AbortGuard {
        AbortGuard {
            previous_val: set_abort(true),
            _not_send: PhantomData,
        }
    }
}

impl Drop for AbortGuard {
    fn drop(&mut self) {
        set_abort(self.previous_val);
    }
}

/// Function being called when a panic happens.
fn panic_hook(info: &PanicInfo, report_url: &'static str) {
    let location = info.location();
    let file = location.as_ref().map(|l| l.file()).unwrap_or("<unknown>");
    let line = location.as_ref().map(|l| l.line()).unwrap_or(0);

    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        }
    };

    let thread = thread::current();
    let name = thread.name().unwrap_or("<unnamed>");

    let backtrace = Backtrace::new();
    let mut stderr = io::stderr();

    let _ = writeln!(stderr, "");
    let _ = writeln!(stderr, "====================");
    let _ = writeln!(stderr, "");
    let _ = writeln!(stderr, "{:?}", backtrace);
    let _ = writeln!(stderr, "");
    let _ = writeln!(
        stderr,
        "Thread '{}' panicked at '{}', {}:{}",
        name, msg, file, line
    );
    let _ = writeln!(stderr, ABOUT_PANIC!(), report_url);
    push_alert_to_ding(format!("{:?}", backtrace));
    ABORT.with(|flag| {
        if flag.get() {
            ::std::process::exit(1);
        }
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PushMsg {
    pub msgtype: String,
    pub text: PushText,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PushText {
    pub content: String
}

pub fn push_alert_to_ding(trace_err: String) {
    let client = reqwest::Client::new();
    let ding_talk_endpoint = "https://oapi.dingtalk.com/robot/send?access_token=";
    let ding_talk_token = env::var("DING_TALK_TOKEN").unwrap_or_default();
    let mut r = client.post(&format!("{}{}", ding_talk_endpoint, ding_talk_token));
    let msg_body = PushMsg { msgtype: "text".to_string(), text: PushText { content: trace_err } };
    let res = r.json(&msg_body).send();
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn does_not_abort() {
		set("test");
		let _guard = AbortGuard::force_unwind();
		::std::panic::catch_unwind(|| panic!()).ok();
	}
}