/*
 * JNI Bridge for Egor Mobile Demo
 *
 * Connects Kotlin/Java to egor_mobile and the bouncing boxes demo
 */

#include <jni.h>
#include <android/native_window.h>
#include <android/native_window_jni.h>
#include <android/log.h>

#include "egor_mobile.h"
#include "bouncing_boxes.h"

#define LOG_TAG "EgorDemo"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

static ANativeWindow* g_window = nullptr;

extern "C" {

JNIEXPORT jint JNICALL
Java_com_egor_demo_EgorSurfaceView_nativeInit(
        JNIEnv* env,
        jobject /* this */,
        jobject surface,
        jint width,
        jint height) {

    LOGI("nativeInit: %dx%d", width, height);

    // Get the native window from the Surface
    g_window = ANativeWindow_fromSurface(env, surface);
    if (!g_window) {
        LOGE("Failed to get ANativeWindow from surface");
        return 0;
    }

    // Initialize egor with the window
    int result = egor_init(g_window, (uint32_t)width, (uint32_t)height);
    if (result != 1) {
        LOGE("egor_init failed");
        ANativeWindow_release(g_window);
        g_window = nullptr;
        return 0;
    }

    LOGI("egor initialized successfully");
    return 1;
}

JNIEXPORT void JNICALL
Java_com_egor_demo_EgorSurfaceView_nativeDemoInit(
        JNIEnv* env,
        jobject /* this */,
        jint width,
        jint height) {

    LOGI("nativeDemoInit: %dx%d", width, height);
    demo_init((uint32_t)width, (uint32_t)height);
}

JNIEXPORT jint JNICALL
Java_com_egor_demo_EgorSurfaceView_nativeDemoFrame(
        JNIEnv* env,
        jobject /* this */,
        jfloat deltaMs) {

    return demo_frame(deltaMs);
}

JNIEXPORT void JNICALL
Java_com_egor_demo_EgorSurfaceView_nativeDemoResize(
        JNIEnv* env,
        jobject /* this */,
        jint width,
        jint height) {

    LOGI("nativeDemoResize: %dx%d", width, height);
    demo_resize((uint32_t)width, (uint32_t)height);
}

JNIEXPORT void JNICALL
Java_com_egor_demo_EgorSurfaceView_nativeDemoTouch(
        JNIEnv* env,
        jobject /* this */,
        jfloat x,
        jfloat y) {

    demo_touch(x, y);
}

JNIEXPORT void JNICALL
Java_com_egor_demo_EgorSurfaceView_nativeDemoCleanup(
        JNIEnv* env,
        jobject /* this */) {

    LOGI("nativeDemoCleanup");
    demo_cleanup();

    if (g_window) {
        ANativeWindow_release(g_window);
        g_window = nullptr;
    }
}

} // extern "C"
