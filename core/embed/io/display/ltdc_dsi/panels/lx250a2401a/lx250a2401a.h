/*
 * This file is part of the Trezor project, https://trezor.io/
 *
 * Copyright (c) SatoshiLabs
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#pragma once

#include <trezor_types.h>

typedef struct {
  uint64_t dsi_lane_byte_freq_hz;
  uint32_t pll_dsi_ndiv;
  uint32_t pll_dsi_odf;
  uint32_t dsi_dphy_frange;
  uint32_t dsi_tx_escape_clk_div;

  uint32_t phy_timer_clk_hs2lp;
  uint32_t phy_timer_clk_lp2hs;
  uint32_t phy_timer_data_hs2lp;
  uint32_t phy_timer_data_lp2hs;

  uint64_t ltdc_pixel_clock_hz;
  uint32_t pll3_n;
  uint32_t pll3_r;

  float dsi_byte_clk_to_pixel_clk_ratio;

  uint32_t vfp[6];
  uint32_t hfp;
  uint32_t hact;
  uint32_t lcd_width;

  uint32_t dsi_pixel_format;
} display_configuration_t;

#define VSYNC 2
#define VBP 26
#define VACT 520
#define HSYNC 6
#define HBP 2
#define LCD_HEIGHT 520

#define LCD_Y_OFFSET 0
#define LCD_X_OFFSET 50

#define GFXMMU_LUT_FIRST 0
#define GFXMMU_LUT_LAST 519
#define GFXMMU_LUT_SIZE 520

#define PANEL_DSI_MODE DSI_VID_MODE_BURST
#define PANEL_DSI_LANES DSI_TWO_DATA_LANES
#define PANEL_LTDC_PIXEL_FORMAT LTDC_PIXEL_FORMAT_ARGB8888

// IMPORTANT:
//
// Changing this value affects constants in backlight.rs and bootui.h
// (for example: BACKLIGHT_NORMAL, BACKLIGHT_LOW, BACKLIGHT_DIM,
// BACKLIGHT_NONE, BACKLIGHT_MIN, and BACKLIGHT_MAX). Ensure these
// values remain consistent.
// Additionally, changing this value can affect CI tests, production-
// line tests, and backlight settings on devices in the field.
//
// See issue #6028 for details.
#define GAMMA_EXP 2.2f

// Size of the physical frame buffer in bytes
//
// It's smaller than size of the virtual frame buffer
// due to used GFXMMU settings
#define PHYSICAL_FRAME_BUFFER_SIZE (765 * 1024)

// Pitch (in pixels) of the virtual frame buffer
#define FRAME_BUFFER_PIXELS_PER_LINE 768

#define VIRTUAL_FRAME_BUFFER_SIZE \
  (FRAME_BUFFER_PIXELS_PER_LINE * LCD_HEIGHT * 4)
