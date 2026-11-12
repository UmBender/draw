#!/usr/bin/env bash
# The macroquad and miniquad items draw really uses (T127 baseline).
# Usage: scripts/audit/mq-calls.sh
set -eu # no pipefail: grep exits 1 on "no match", which is a valid count of 0
cd "$(dirname "$0")/../.."

echo "## macroquad paths imported in src"
echo
echo '```text'
grep -rhoE 'macroquad::[A-Za-z_:{}, ]+' src | tr -d ' ' | sort | uniq -c | sort -rn
echo '```'
echo
echo "## miniquad paths used directly in src"
echo
echo '```text'
grep -rhoE 'miniquad::[A-Za-z_:{}, ]+' src | tr -d ' ' | sort | uniq -c | sort -rn || true
echo '```'
echo
echo "## Frequent macroquad free functions called"
echo
echo "| Call | Occurrences |"
echo "|------|-------------|"
for fn in draw_triangle draw_line draw_rectangle draw_circle draw_texture_ex draw_texture \
  draw_text_ex draw_mesh clear_background next_frame get_time get_frame_time \
  mouse_position is_key_down is_key_pressed is_mouse_button_down get_char_pressed \
  screen_width screen_height set_camera set_default_camera render_target \
  load_texture_from_image get_clipboard set_clipboard request_new_screen_size; do
  n=$(grep -rwo "$fn" src | wc -l)
  if [ "$n" -gt 0 ]; then echo "| \`$fn\` | $n |"; fi
done
