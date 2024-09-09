//! # Modem helper functions for nrfxlib
//!
//! Helper functions for dealing with the LTE modem.
//!
//! Copyright (c) 42 Technology Ltd 2019
//!
//! Dual-licensed under MIT and Apache 2.0. See the [README](../README.md) for
//! more details.

//******************************************************************************
// Sub-Modules
//******************************************************************************

// None

//******************************************************************************
// Imports
//******************************************************************************

use crate::Error;
use log::debug;

//******************************************************************************
// Types
//******************************************************************************

/// Identifies which radios in the nRF9160 should be active
#[derive(Debug, Copy, Clone)]
pub enum SystemMode {
	/// LTE-M only
	LteM,
	/// NB-IoT only
	NbIot,
	/// GNSS Only
	GnssOnly,
	/// LTE-M and GNSS
	LteMAndGnss,
	/// NB-IOT and GNSS
	NbIotAndGnss,
}

//******************************************************************************
// Constants
//******************************************************************************

// None

//******************************************************************************
// Global Variables
//******************************************************************************

// None

//******************************************************************************
// Macros
//******************************************************************************

// None

//******************************************************************************
// Public Functions and Impl on Public Types
//******************************************************************************

/// Waits for the modem to connect to a network.
///
/// The list of acceptable CEREG response indications is taken from the Nordic
/// `lte_link_control` driver.
pub fn wait_for_lte() -> Result<(), Error> {
	debug!("Waiting for LTE...");
	let skt = crate::at::AtSocket::new()?;
	// Subscribe
	skt.write(b"AT+CEREG=2")?;

	let connected_indications = ["+CEREG: 1", "+CEREG:1", "+CEREG: 5", "+CEREG:5"];
	'outer: loop {
		let mut buf = [0u8; 128];
		let maybe_length = skt.recv(&mut buf)?;
		if let Some(length) = maybe_length {
			let s = unsafe { core::str::from_utf8_unchecked(&buf[0..length - 1]) };
			for line in s.lines() {
				let line = line.trim();
				debug!("RX {:?}", line);
				for ind in &connected_indications {
					if line.starts_with(ind) {
						break 'outer;
					}
				}
			}
		} else {
			cortex_m::asm::wfe();
		}
	}
	Ok(())
}

/// Powers the modem on and sets it to auto-register, but does not wait for it
/// to connect to a network.
pub fn on() -> Result<(), Error> {
	debug!("Turning modem ON");
	crate::at::send_at_command("AT+CFUN=1", |_| {})?;
	Ok(())
}

/// Puts the modem into flight mode.
pub fn flight_mode() -> Result<(), Error> {
	debug!("Turning mode to FLIGHT MODE");
	crate::at::send_at_command("AT+CFUN=4", |_| {})?;
	Ok(())
}

/// Powers the modem off.
pub fn off() -> Result<(), Error> {
	debug!("Turning modem OFF");
	crate::at::send_at_command("AT+CFUN=0", |_| {})?;
	Ok(())
}

/// Set which radios should be active. Only works when modem is off.
pub fn set_system_mode(mode: SystemMode) -> Result<(), Error> {
	let at_command = match mode {
		SystemMode::LteM => "AT%XSYSTEMMODE=1,0,0,0",
		SystemMode::NbIot => "AT%XSYSTEMMODE=0,1,0,0",
		SystemMode::GnssOnly => "AT%XSYSTEMMODE=0,0,1,0",
		SystemMode::LteMAndGnss => "AT%XSYSTEMMODE=1,0,1,0",
		SystemMode::NbIotAndGnss => "AT%XSYSTEMMODE=0,1,1,0",
	};
	debug!("{:?} => {:?}", mode, at_command);
	crate::at::send_at_command(at_command, |_| {})?;
	Ok(())
}

/// Get which radios should be active
pub fn get_system_mode() -> Result<SystemMode, Error> {
	let mut result = Err(Error::UnrecognisedValue);
	// Don't care about final digit - that's just the LTE/NB-IOT preference
	crate::at::send_at_command("AT%XSYSTEMMODE?", |res| {
		if res.starts_with("%XSYSTEMMODE: 1,0,0,") {
			result = Ok(SystemMode::LteM);
		} else if res.starts_with("%XSYSTEMMODE: 0,1,0,") {
			result = Ok(SystemMode::NbIot);
		} else if res.starts_with("%XSYSTEMMODE: 0,0,1,") {
			result = Ok(SystemMode::GnssOnly);
		} else if res.starts_with("%XSYSTEMMODE: 1,0,1,") {
			result = Ok(SystemMode::LteMAndGnss);
		} else if res.starts_with("%XSYSTEMMODE: 0,1,1,") {
			result = Ok(SystemMode::NbIotAndGnss);
		}
		debug!("{:?} => {:?}", res, result);
	})?;
	result
}
