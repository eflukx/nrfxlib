//! # Clock for nrfxlib
//!
//! nrfxlib has no time source of its own. The application registers one with [`set_clock`]; it's
//! used for timeouts while waiting for the modem, and to sleep (instead of busy-wait) meanwhile.
//! Without a registered clock there are no timeouts and waiting spins, as before.
//!
//! Dual-licensed under MIT and Apache 2.0. See the [README](../README.md) for
//! more details.

use crate::Error;
use core::cell::Cell;
use cortex_m::interrupt::Mutex;

/// Time source and low power wait, provided by the application.
#[derive(Debug, Clone, Copy)]
pub struct Clock {
	/// Monotonic time in milliseconds (e.g. since boot).
	pub now_ms: fn() -> u64,
	/// Sleep until an event or interrupt occurs (e.g. from the modem), or `deadline_ms` has passed,
	/// whichever comes first. May return early: callers check their condition and deadline again.
	pub wait_for_event_until_ms: fn(deadline_ms: u64),
}

static CLOCK: Mutex<Cell<Option<Clock>>> = Mutex::new(Cell::new(None));

/// Register the clock nrfxlib uses for timeouts and low power waiting.
pub fn set_clock(clock: Clock) {
	cortex_m::interrupt::free(|cs| CLOCK.borrow(cs).set(Some(clock)));
}

/// The registered clock, if any.
pub(crate) fn clock() -> Option<Clock> {
	cortex_m::interrupt::free(|cs| CLOCK.borrow(cs).get())
}

/// A point in time to wait for events until. Never expires without a registered clock.
pub(crate) struct Deadline(Option<(Clock, u64)>);

impl Deadline {
	/// Deadline `timeout_ms` from now.
	pub(crate) fn after_ms(timeout_ms: u64) -> Self {
		Deadline(clock().map(|clock| (clock, (clock.now_ms)().saturating_add(timeout_ms))))
	}

	/// Wait for an event (e.g. a modem interrupt) or the deadline. [`Error::Timeout`] once the
	/// deadline has passed.
	pub(crate) fn wait(&self) -> Result<(), Error> {
		match self.0 {
			Some((clock, deadline_ms)) if (clock.now_ms)() >= deadline_ms => Err(Error::Timeout),
			Some((clock, deadline_ms)) => {
				(clock.wait_for_event_until_ms)(deadline_ms);
				Ok(())
			}
			None => Ok(()),
		}
	}
}
