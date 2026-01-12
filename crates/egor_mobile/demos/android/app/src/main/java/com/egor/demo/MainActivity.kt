package com.egor.demo

import android.graphics.PixelFormat
import android.os.Bundle
import android.view.MotionEvent
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.WindowManager
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    private lateinit var surfaceView: EgorSurfaceView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Fullscreen
        window.addFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN)
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

        surfaceView = EgorSurfaceView(this)
        setContentView(surfaceView)
    }

    override fun onDestroy() {
        surfaceView.cleanup()
        super.onDestroy()
    }
}

class EgorSurfaceView(context: MainActivity) : SurfaceView(context), SurfaceHolder.Callback, Runnable {
    private var renderThread: Thread? = null
    private var running = false
    private var lastFrameTime = System.nanoTime()

    init {
        // Set pixel format for OpenGL ES compatibility
        holder.setFormat(PixelFormat.RGBA_8888)
        holder.addCallback(this)
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        // Initialize egor with the surface
        val surface = holder.surface
        val result = nativeInit(surface, width, height)
        if (result != 1) {
            throw RuntimeException("Failed to initialize egor")
        }

        // Initialize demo
        nativeDemoInit(width, height)

        // Start render thread
        running = true
        renderThread = Thread(this)
        renderThread?.start()
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        nativeDemoResize(width, height)
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        running = false
        renderThread?.join()
        renderThread = null
    }

    override fun run() {
        while (running) {
            val currentTime = System.nanoTime()
            val deltaMs = (currentTime - lastFrameTime) / 1_000_000f
            lastFrameTime = currentTime

            nativeDemoFrame(deltaMs)

            // Cap at ~60fps
            val frameTime = (System.nanoTime() - currentTime) / 1_000_000f
            if (frameTime < 16.67f) {
                Thread.sleep((16.67f - frameTime).toLong())
            }
        }
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        when (event.action) {
            MotionEvent.ACTION_DOWN, MotionEvent.ACTION_MOVE -> {
                nativeDemoTouch(event.x, event.y)
                return true
            }
        }
        return super.onTouchEvent(event)
    }

    fun cleanup() {
        running = false
        renderThread?.join()
        nativeDemoCleanup()
    }

    // Native methods - implemented in C++
    private external fun nativeInit(surface: Surface, width: Int, height: Int): Int
    private external fun nativeDemoInit(width: Int, height: Int)
    private external fun nativeDemoFrame(deltaMs: Float): Int
    private external fun nativeDemoResize(width: Int, height: Int)
    private external fun nativeDemoTouch(x: Float, y: Float)
    private external fun nativeDemoCleanup()

    companion object {
        init {
            // Load egor_mobile first (dependency), then the demo
            System.loadLibrary("egor_mobile")
            System.loadLibrary("egor_demo")
        }
    }
}
