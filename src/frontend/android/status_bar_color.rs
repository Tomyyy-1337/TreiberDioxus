use std::mem::ManuallyDrop;

use jni::objects::{JObject, JValue};
use jni::{jni_sig, jni_str};

/// Sets the Android status bar colour (`argb` as `0xAARRGGBB`).
pub fn set_status_bar_color(argb: u32) -> jni::errors::Result<()> {
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

        env.call_method(
            &window,
            jni_str!("setStatusBarColor"),
            jni_sig!("(I)V"),
            &[JValue::Int(argb as i32)],
        )?;
        Ok(())
    })
}
