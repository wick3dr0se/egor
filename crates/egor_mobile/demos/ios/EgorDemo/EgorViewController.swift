//
//  EgorViewController.swift
//  EgorDemo
//
//  Metal-based view controller for egor rendering
//

import UIKit
import MetalKit
import QuartzCore

class EgorViewController: UIViewController {

    private var metalView: EgorMetalView!

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .black

        // Create Metal view
        metalView = EgorMetalView(frame: view.bounds)
        metalView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        view.addSubview(metalView)
    }

    override var prefersStatusBarHidden: Bool {
        return true
    }

    override var prefersHomeIndicatorAutoHidden: Bool {
        return true
    }
}

class EgorMetalView: UIView {

    private var displayLink: CADisplayLink?
    private var lastFrameTime: CFTimeInterval = 0
    private var initialized = false

    override class var layerClass: AnyClass {
        return CAMetalLayer.self
    }

    private var metalLayer: CAMetalLayer {
        return layer as! CAMetalLayer
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        setupMetal()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        setupMetal()
    }

    private func setupMetal() {
        // Configure metal layer
        metalLayer.device = MTLCreateSystemDefaultDevice()
        metalLayer.pixelFormat = .bgra8Unorm
        metalLayer.framebufferOnly = true
        metalLayer.contentsScale = UIScreen.main.scale

        // Enable touch
        isUserInteractionEnabled = true
        isMultipleTouchEnabled = true
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()

        if window != nil && !initialized {
            initializeEgor()
        }
    }

    private func initializeEgor() {
        let scale = UIScreen.main.scale
        let width = UInt32(bounds.width * scale)
        let height = UInt32(bounds.height * scale)

        // Get pointer to CAMetalLayer
        let layerPtr = Unmanaged.passUnretained(metalLayer).toOpaque()

        // Initialize egor
        let result = egor_init(layerPtr, width, height)
        guard result == 1 else {
            print("Failed to initialize egor")
            return
        }

        // Initialize demo
        demo_init(width, height)

        initialized = true

        // Start render loop
        displayLink = CADisplayLink(target: self, selector: #selector(render))
        displayLink?.preferredFrameRateRange = CAFrameRateRange(minimum: 30, maximum: 120, preferred: 60)
        displayLink?.add(to: .main, forMode: .common)

        lastFrameTime = CACurrentMediaTime()

        print("Egor initialized: \(width)x\(height)")
    }

    @objc private func render() {
        guard initialized else { return }

        let currentTime = CACurrentMediaTime()
        let deltaMs = Float((currentTime - lastFrameTime) * 1000.0)
        lastFrameTime = currentTime

        _ = demo_frame(deltaMs)
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        guard initialized else { return }

        let scale = UIScreen.main.scale
        let width = UInt32(bounds.width * scale)
        let height = UInt32(bounds.height * scale)

        demo_resize(width, height)
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        handleTouches(touches)
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        handleTouches(touches)
    }

    private func handleTouches(_ touches: Set<UITouch>) {
        guard initialized else { return }

        let scale = UIScreen.main.scale
        for touch in touches {
            let location = touch.location(in: self)
            demo_touch(Float(location.x * scale), Float(location.y * scale))
        }
    }

    deinit {
        displayLink?.invalidate()
        if initialized {
            demo_cleanup()
        }
    }
}
