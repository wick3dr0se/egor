/*
 * Bouncing Boxes Demo for egor_mobile
 *
 * Colorful rectangles bouncing around with physics!
 */

#include "bouncing_boxes.h"
#include "egor_mobile.h"
#include <stdlib.h>
#include <math.h>
#include <time.h>

#define MAX_BOXES 100
#define BOX_SIZE 60.0f
#define GRAVITY 500.0f
#define BOUNCE_DAMPING 0.8f
#define INITIAL_BOXES 5

typedef struct {
    float x, y;
    float vx, vy;
    float r, g, b;
    float rotation;
    float rotation_speed;
} Box;

static Box boxes[MAX_BOXES];
static int box_count = 0;
static uint32_t screen_width = 800;
static uint32_t screen_height = 600;
static bool initialized = false;

static float randf(void) {
    return (float)rand() / (float)RAND_MAX;
}

static void add_box(float x, float y) {
    if (box_count >= MAX_BOXES) return;

    Box* b = &boxes[box_count++];
    b->x = x - BOX_SIZE / 2;
    b->y = y - BOX_SIZE / 2;
    b->vx = (randf() - 0.5f) * 400.0f;
    b->vy = (randf() - 0.5f) * 200.0f - 200.0f; // Initial upward velocity
    b->r = 0.3f + randf() * 0.7f;
    b->g = 0.3f + randf() * 0.7f;
    b->b = 0.3f + randf() * 0.7f;
    b->rotation = 0.0f;
    b->rotation_speed = (randf() - 0.5f) * 5.0f;
}

void demo_init(uint32_t width, uint32_t height) {
    srand((unsigned int)time(NULL));

    screen_width = width;
    screen_height = height;
    box_count = 0;

    // Set a nice dark background
    egor_set_clear_color(0.1f, 0.1f, 0.15f, 1.0f);

    // Spawn initial boxes in the center
    for (int i = 0; i < INITIAL_BOXES; i++) {
        add_box(width / 2.0f + (randf() - 0.5f) * 200.0f,
                height / 3.0f);
    }

    initialized = true;
}

int32_t demo_frame(float delta_ms) {
    if (!initialized) return 0;

    float dt = delta_ms / 1000.0f; // Convert to seconds

    // Update physics for each box
    for (int i = 0; i < box_count; i++) {
        Box* b = &boxes[i];

        // Apply gravity
        b->vy += GRAVITY * dt;

        // Update position
        b->x += b->vx * dt;
        b->y += b->vy * dt;

        // Update rotation
        b->rotation += b->rotation_speed * dt;

        // Bounce off walls
        if (b->x < 0) {
            b->x = 0;
            b->vx = -b->vx * BOUNCE_DAMPING;
            b->rotation_speed = -b->rotation_speed;
        }
        if (b->x + BOX_SIZE > screen_width) {
            b->x = screen_width - BOX_SIZE;
            b->vx = -b->vx * BOUNCE_DAMPING;
            b->rotation_speed = -b->rotation_speed;
        }

        // Bounce off floor
        if (b->y + BOX_SIZE > screen_height) {
            b->y = screen_height - BOX_SIZE;
            b->vy = -b->vy * BOUNCE_DAMPING;

            // Add some friction
            b->vx *= 0.99f;

            // Reduce rotation when on ground
            b->rotation_speed *= 0.95f;
        }

        // Bounce off ceiling
        if (b->y < 0) {
            b->y = 0;
            b->vy = -b->vy * BOUNCE_DAMPING;
        }

        // Draw the box (using center position for proper rotation)
        // Note: egor_draw_rect uses top-left position
        egor_draw_rect(
            b->x, b->y,
            BOX_SIZE, BOX_SIZE,
            b->r, b->g, b->b, 1.0f,
            0  // No texture
        );
    }

    // Draw "Tap to add boxes!" text hint using small rectangles
    // (Simple text replacement since we don't have text rendering in FFI yet)
    float hint_y = 30.0f;
    float hint_x = screen_width / 2.0f - 80.0f;
    for (int i = 0; i < 10; i++) {
        egor_draw_rect(
            hint_x + i * 18.0f, hint_y,
            12.0f, 4.0f,
            0.5f, 0.5f, 0.5f, 0.3f,
            0
        );
    }

    // Render the frame
    return egor_render_frame(delta_ms);
}

void demo_resize(uint32_t width, uint32_t height) {
    screen_width = width;
    screen_height = height;
    egor_resize(width, height);
}

void demo_touch(float x, float y) {
    add_box(x, y);
}

void demo_cleanup(void) {
    box_count = 0;
    initialized = false;
    egor_cleanup();
}
