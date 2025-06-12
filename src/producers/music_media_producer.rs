use std::{ffi::c_void, ptr};

use chrono::Utc;
use colored::Colorize;
use core_foundation::{
    base::TCFType, dictionary::CFDictionaryRef, runloop::CFRunLoopRun, string::CFString,
};
use core_foundation_sys::notification_center::{
    CFNotificationCenterAddObserver, CFNotificationCenterGetDistributedCenter,
    CFNotificationCenterRef, CFNotificationName,
    CFNotificationSuspensionBehaviorDeliverImmediately,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{media_event::MediaEvent, medias::music_media::MusicMedia, voidp_to_ref};

extern "C" fn music_event_handler(
    _: CFNotificationCenterRef,
    tx_pointer: *mut c_void,
    _: CFNotificationName,
    _: *const c_void,
    system_event: CFDictionaryRef,
) {
    unsafe {
        let event = MusicMedia::from_cf_dictionary(system_event);
        let tx_ref: &UnboundedSender<MediaEvent> = voidp_to_ref(tx_pointer);
        let media_event = MediaEvent::Music {
            media: event,
            emitted_at: Utc::now().timestamp_millis(),
        };
        if let Err(e) = tx_ref.send(media_event) {
            println!("{} Sending failed to channel! {}", "[error]".red(), e)
        }
    }
}

pub fn relay_media_events(tx: UnboundedSender<MediaEvent>) {
    //Registering handler for DistributedNotificationCenter and kicking off the run loop
    unsafe {
        let nc = CFNotificationCenterGetDistributedCenter();

        CFNotificationCenterAddObserver(
            nc,
            ptr::addr_of!(tx) as *const _, // The transmitter for channel is passed directly to the handler
            music_event_handler,
            CFString::new("com.apple.Music.playerInfo").as_concrete_TypeRef(),
            ptr::null(),
            CFNotificationSuspensionBehaviorDeliverImmediately,
        );
        CFRunLoopRun();
    }
}
