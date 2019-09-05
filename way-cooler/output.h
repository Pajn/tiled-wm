#ifndef WC_OUTPUT_H
#define WC_OUTPUT_H

#include <wayland-server.h>
#include <wlr/types/wlr_box.h>
#include <wlr/types/wlr_output.h>
#include <wlr/types/wlr_output_damage.h>

#include "server.h"

void wc_output_init(struct wc_server *server);

void wc_output_fini(struct wc_server *server);

// Gets the output that was last active (e.g. last had user activity).
//
// If there are no outputs, NULL is returned. If there has been no activity,
// the first output in the list is returned.
struct Output *wc_get_active_output(struct wc_server *server);

/// Damages the surface which is at the given output coordinates.
///
/// If surface_damage is NULL the entire surface is damaged using the
/// geometry provided in surface_output_geo.
void wc_output_damage_surface(struct Output *output,
		struct wlr_surface *surface, pixman_region32_t *surface_damage,
		struct wlr_box surface_output_geo);

#endif  // WC_OUTPUT_H
