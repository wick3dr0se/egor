/*
 * Bouncing Boxes Demo for egor_mobile
 *
 * A simple demo showing animated rectangles bouncing around the screen.
 * Touch to add more boxes!
 */

#ifndef BOUNCING_BOXES_H
#define BOUNCING_BOXES_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Initialize the demo with screen dimensions
void demo_init(uint32_t width, uint32_t height);

// Update and render a frame, returns 1 on success
int32_t demo_frame(float delta_ms);

// Handle screen resize
void demo_resize(uint32_t width, uint32_t height);

// Handle touch to add a new box at that position
void demo_touch(float x, float y);

// Clean up demo resources
void demo_cleanup(void);

#ifdef __cplusplus
}
#endif

#endif /* BOUNCING_BOXES_H */
