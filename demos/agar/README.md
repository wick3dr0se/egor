# Egor Agar Demo

A version of the classic [agar.io](https://agar.io) built as a demo for **egor**

![Agar Screenshot](/media/agar.png)

## How to Play

Move your cell by pointing your mouse in the direction you want to go. Eat food (red) and smaller creatures (orange) to grow. Don't get eaten by anything bigger than you.

- Absorb food and creatures smaller than you to grow
- Larger cells move slower
- Last one standing wins, you lose if any creature outlives you
- Spectate after death

## Controls

| Input           | Action      |
| --------------- | ----------- |
| Cursor position | Move cell   |
| Scroll wheel    | Zoom in/out |

## Comparison

I wrote a mirror of this game/demo in macroquad previously here: https://github.com/wick3dr0se/agar

In this egor version we get higher framerate and no rendering distortions with smoother geometry (circles have specified 32 segments)
