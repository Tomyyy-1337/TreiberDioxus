use std::mem::ManuallyDrop;

use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

/// `requestedOrientation` value that locks the activity to portrait.
pub const SCREEN_ORIENTATION_PORTRAIT: i32 = 1;

/// Sets the activity's `requestedOrientation` (see `SCREEN_ORIENTATION_PORTRAIT`, `6` sensor landscape).
pub fn set_requested_orientation(orientation: i32) -> jni::errors::Result<()> {
    let ctx = ndk_context::android_context();
    // SAFETY: ndk-context guarantees the VM pointer stays valid for the process lifetime.
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) };

    vm.attach_current_thread(|env| {
        // SAFETY: the activity is owned by the JVM; ManuallyDrop stops this wrapper deleting it.
        let activity = ManuallyDrop::new(unsafe { JObject::from_raw(env, ctx.context().cast()) });

        env.call_method(
            &*activity,
            jni_str!("setRequestedOrientation"),
            jni_sig!("(I)V"),
            &[JValue::Int(orientation)],
        )?;
        Ok(())
    })
}

/// Hides or shows the status and navigation bars through the window's insets controller.
pub fn set_system_bars_hidden(hidden: bool) -> jni::errors::Result<()> {
    let ctx = ndk_context::android_context();
    // SAFETY: ndk-context guarantees the VM pointer stays valid for the process lifetime.
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) };

    vm.attach_current_thread(|env| {
        // SAFETY: the activity is owned by the JVM; ManuallyDrop stops this wrapper deleting it.
        let activity = ManuallyDrop::new(unsafe { JObject::from_raw(env, ctx.context().cast()) });

        let window = env
            .call_method(
                &*activity,
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;
        let controller = env
            .call_method(
                &window,
                jni_str!("getInsetsController"),
                jni_sig!("()Landroid/view/WindowInsetsController;"),
                &[],
            )?
            .l()?;

        let types = env.find_class(jni_str!("android/view/WindowInsets$Type"))?;
        let status = env
            .call_static_method(&types, jni_str!("statusBars"), jni_sig!("()I"), &[])?
            .i()?;
        let navigation = env
            .call_static_method(&types, jni_str!("navigationBars"), jni_sig!("()I"), &[])?
            .i()?;
        let mask = status | navigation;

        if hidden {
            // BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE = 2
            env.call_method(
                &controller,
                jni_str!("setSystemBarsBehavior"),
                jni_sig!("(I)V"),
                &[JValue::Int(2)],
            )?;
            env.call_method(
                &controller,
                jni_str!("hide"),
                jni_sig!("(I)V"),
                &[JValue::Int(mask)],
            )?;
        } else {
            env.call_method(
                &controller,
                jni_str!("show"),
                jni_sig!("(I)V"),
                &[JValue::Int(mask)],
            )?;
        }
        Ok(())
    })
}
