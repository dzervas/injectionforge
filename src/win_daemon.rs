#![cfg(windows)]

use std::ffi::c_void;
use winapi::shared::minwindef::DWORD;
use winapi::um::evntprov::*;
use winapi::um::evntcons::*;
use winapi::um::evntprov::*;
use winapi::um::winnt::{EVENT_TRACE_CONTROL_STOP, EVENT_TRACE_FLAG_PROCESS};

pub fn start_daemon() {
	// Create an event trace session
	let session_name = "InjectionForge";
	let session_handle = create_event_trace_session(session_name);
	if session_handle.is_null() {
		eprintln!("Failed to create event trace session");
		return;
	}

	// Enable process creation events
	enable_process_creation_events(session_handle);

	// Process events until a termination event is received
	process_events(session_handle);

	// Stop the event trace session
	stop_event_trace_session(session_handle);
}

fn create_event_trace_session(session_name: &str) -> TRACEHANDLE {
	let session_name = widestring::WideCString::from_str(session_name).expect("Failed to convert session name");

	let mut session_handle: TRACEHANDLE = 0;
	let status = unsafe {
		StartTraceW(
			&mut session_handle,
			session_name.as_ptr(),
			ptr::null_mut(),
		)
	};

	if status != ERROR_SUCCESS {
		println!("Failed to start event trace session: {}", status);
	}

	session_handle
}

fn enable_process_creation_events(session_handle: TRACEHANDLE) {
	let status = unsafe {
		EnableTraceEx2(
			session_handle,
			&EVENT_TRACE_GUID_PROCESS,
			EVENT_CONTROL_CODE_ENABLE_PROVIDER,
			TRACE_LEVEL_INFORMATION,
			EVENT_TRACE_FLAG_PROCESS,
			0,
			0,
			0,
			NULL,
		)
	};

	if status != ERROR_SUCCESS {
		println!("Failed to enable process creation events: {}", status);
	}
}

fn process_events(session_handle: TRACEHANDLE) {
	let mut buffer_size: DWORD = 64 * 1024;
	let mut buffer = vec![0u8; buffer_size as usize];

	let status = unsafe {
		ProcessTrace(
			&mut session_handle,
			1,
			NULL,
			NULL,
		)
	};

	if status != ERROR_SUCCESS && status != ERROR_CANCELLED {
		println!("Failed to process events: {}", status);
	}
}

fn stop_event_trace_session(session_handle: TRACEHANDLE) {
	let status = unsafe {
		ControlTraceW(
			session_handle,
			NULL,
			NULL,
			EVENT_TRACE_CONTROL_STOP,
		)
	};

	if status != ERROR_SUCCESS {
		println!("Failed to stop event trace session: {}", status);
	}
}
