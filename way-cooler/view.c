#include "view.h"

#include <assert.h>
#include <stdlib.h>

#include <wayland-server.h>
#include <wlr/types/wlr_output_damage.h>
#include <wlr/types/wlr_surface.h>
#include <wlr/util/log.h>

#include "cursor.h"
#include "layer_shell.h"
#include "output.h"
#include "seat.h"
#include "server.h"
#include "xdg.h"
#include "xwayland.h"

struct View* get_view_by_id(struct wc_server *server, Id window_id) {
	struct View *view;
	wl_list_for_each(view, &server->views, link) {
		if (view->window_id == window_id) {
			return view;
		}
	}
	assert(false);
}

static bool wc_is_view_at(struct View *view, double lx, double ly,
		double *out_sx, double *out_sy, struct wlr_surface **out_surface) {
	int view_sx = lx - view->geo.x;
	int view_sy = ly - view->geo.y;

	*out_surface = NULL;
	switch (view->surface_type.tag) {
	case Xdg:
		*out_surface = wlr_xdg_surface_surface_at(
				view->surface_type.xdg.xdg_surface, view_sx, view_sy, out_sx, out_sy);
		break;
	case Xwayland:
		if (view->surface_type.xwayland.xwayland_surface->surface != NULL) {
			*out_surface =
					wlr_surface_surface_at(view->surface_type.xwayland.xwayland_surface->surface,
							view_sx, view_sy, out_sx, out_sy);
		}
		break;
	}
	return *out_surface != NULL;
}

void wc_view_get_outputs(struct wlr_output_layout *layout, struct View *view,
		struct wlr_output **out_outputs) {
	struct wlr_box geo = {0};
	memcpy(&geo, &view->geo, sizeof(struct wlr_box));

	size_t next_index = 0;
	// top left
	out_outputs[next_index++] =
			wlr_output_layout_output_at(layout, geo.x, geo.y);
	// top right
	out_outputs[next_index++] =
			wlr_output_layout_output_at(layout, geo.x + geo.width, geo.y);
	// bottom left
	out_outputs[next_index++] =
			wlr_output_layout_output_at(layout, geo.x, geo.y + geo.height);
	// bottom right
	out_outputs[next_index++] = wlr_output_layout_output_at(
			layout, geo.x + geo.width, geo.y + geo.height);

	for (size_t i = 1; i < 4; i++) {
		for (size_t j = 0; j < i; j++) {
			if (out_outputs[i] == out_outputs[j]) {
				out_outputs[i] = NULL;
			}
		}
	}
}

struct wlr_surface *wc_view_surface(struct View *view) {
	switch (view->surface_type.tag) {
	case Xdg:
		return view->surface_type.xdg.xdg_surface->surface;
	case Xwayland:
		return view->surface_type.xwayland.xwayland_surface->surface;
	}
	return NULL;
}

void wc_view_move(struct View *view, struct wlr_box geo) {
	struct wc_server *server = view->server.server;
	struct wlr_surface *focused_surface =
			server->seat->seat->pointer_state.focused_surface;
	struct wlr_surface *surface = wc_view_surface(view);

	if (surface != focused_surface) {
		return;
	}

	struct wc_cursor *cursor = server->cursor;
	struct wlr_cursor *wlr_cursor = cursor->wlr_cursor;

	cursor->cursor_mode = WC_CURSOR_MOVE;
	cursor->grabbed.view = view;
	cursor->grabbed.original_x = wlr_cursor->x - view->geo.x;
	cursor->grabbed.original_y = wlr_cursor->y - view->geo.y;

	cursor->grabbed.original_view_geo.x = view->geo.x;
	cursor->grabbed.original_view_geo.y = view->geo.y;
	cursor->grabbed.original_view_geo.width = geo.width;
	cursor->grabbed.original_view_geo.height = geo.height;
}

void wc_view_resize(struct View *view, struct wlr_box geo, uint32_t edges) {
	struct wc_server *server = view->server.server;
	struct wc_cursor *cursor = server->cursor;
	struct wlr_cursor *wlr_cursor = cursor->wlr_cursor;
	struct wlr_surface *focused_surface =
			server->seat->seat->pointer_state.focused_surface;
	struct wlr_surface *surface = wc_view_surface(view);

	if (surface != focused_surface) {
		return;
	}

	cursor->cursor_mode = WC_CURSOR_RESIZE;
	cursor->grabbed.view = view;
	cursor->grabbed.original_x = wlr_cursor->x;
	cursor->grabbed.original_y = wlr_cursor->y;

	cursor->grabbed.original_view_geo.x = view->geo.x;
	cursor->grabbed.original_view_geo.y = view->geo.y;
	cursor->grabbed.original_view_geo.width = geo.width;
	cursor->grabbed.original_view_geo.height = geo.height;

	cursor->grabbed.resize_edges = edges;
}

void wc_view_update_geometry(struct View *view, struct wlr_box new_geo) {
	switch (view->surface_type.tag) {
	case Xdg:
		view->pending_serial = wlr_xdg_toplevel_set_size(
				view->surface_type.xdg.xdg_surface, new_geo.width, new_geo.height);
		view->is_pending_serial = true;
		break;
	case Xwayland:
		view->pending_serial = 1;
		wlr_xwayland_surface_configure(view->surface_type.xwayland.xwayland_surface, new_geo.x,
				new_geo.y, new_geo.width, new_geo.height);
		break;
	}

	// TODO: Did cursor resize break due to pending_geometry removal?
	// memcpy(&view->pending_geometry, &new_geo, sizeof(struct wlr_box));
	advice_modified_window(view->server.server->wm, view);
}

// void wc_view_update_geometry_from_wm(struct wc_server *server, Id window_id) {
// 	struct View *view = get_view_by_id(server, window_id);
// 	wlr_box new_geo = get_window_geometry(view->server.server->wm, view->window_id);
// 	view->geo = new_geo;
// 	wc_view_damage_whole(view);
// }

void wc_view_damage(struct View *view, pixman_region32_t *damage) {
	struct wlr_output *outputs[4] = {0};
	wc_view_get_outputs(view->server.server->output_layout, view, outputs);

	// Keep a copy of the damage because otherwise it gets screwed up
	// in the presence of multiple outputs.
	pixman_region32_t damage_copy;
	pixman_region32_init(&damage_copy);
	if (damage != NULL) {
		pixman_region32_copy(&damage_copy, damage);
	}
	struct wlr_surface *surface = NULL;
	switch (view->surface_type.tag) {
	case Xdg:
		surface = view->surface_type.xdg.xdg_surface->surface;
		break;
	case Xwayland:
		surface = view->surface_type.xwayland.xwayland_surface->surface;
		break;
	}
	if (surface == NULL) {
		return;
	}

	for (size_t i = 0; i < 4; i++) {
		struct wlr_output *output = outputs[i];
		struct wlr_box *output_box =
				wlr_output_layout_get_box(view->server.server->output_layout, output);

		if (output) {
			struct wlr_box view_output_geo = {
					.x = view->geo.x - output_box->x,
					.y = view->geo.y - output_box->y,
					.width = view->geo.width,
					.height = view->geo.height,
			};
			if (damage != NULL) {
				pixman_region32_translate(
						damage, view_output_geo.x, view_output_geo.y);
				wc_output_damage_surface(
						output->data, surface, damage, view_output_geo);
				pixman_region32_copy(damage, &damage_copy);
			} else {
				wc_output_damage_surface(
						output->data, surface, damage, view_output_geo);
			}
		}
	}
	pixman_region32_fini(&damage_copy);
}

void wc_view_damage_whole(struct View *view) {
	wc_view_damage(view, NULL);
}

void wc_view_commit(struct View *view, struct wlr_box geo) {
	if (!view->mapped) {
		return;
	}

	struct wlr_surface *surface = NULL;
	// struct wlr_box geometry = {0};
	switch (view->surface_type.tag) {
	case Xdg:
		surface = view->surface_type.xdg.xdg_surface->surface;
		// geometry = view->surface_type.xdg.xdg_surface->geometry;
		break;
	case Xwayland:
		surface = view->surface_type.xwayland.xwayland_surface->surface;
		// geometry.height = view->surface_type.xwayland.xwayland_surface->height;
		// geometry.width = view->surface_type.xwayland.xwayland_surface->width;
		break;
	}
	if (surface == NULL) {
		return;
	}

	pixman_region32_t damage;
	pixman_region32_init(&damage);
	wlr_surface_get_effective_damage(surface, &damage);
	wc_view_damage(view, &damage);

	bool size_changed = view->geo.width != surface->current.width ||
			view->geo.height != surface->current.height;

	if (size_changed) {
		wc_view_damage_whole(view);
		view->geo.width = surface->current.width;
		view->geo.height = surface->current.height;
		printf("size_changed\n");
		wc_view_damage_whole(view);
	}

	bool geo_updated = size_changed;

	uint32_t pending_serial = view->pending_serial;
	switch (view->surface_type.tag) {
	case Xdg:
		if (pending_serial > 0 &&
				pending_serial >= view->surface_type.xdg.xdg_surface->configure_serial) {
		// 	wc_view_damage_whole(view);

		// 	if (view->pending_geometry.x != view->geo.x) {
		// printf("pending_geometry x %i::%i\n", view->pending_geometry.x, view->geo.x);
		// 		view->geo.x = view->pending_geometry.x;
		// 		geo_updated = true;
		// 	}
		// 	if (view->pending_geometry.y != view->geo.y) {
		// printf("pending_geometry y %i::%i\n", view->pending_geometry.y, view->geo.y);
		// 		view->geo.y = view->pending_geometry.y;
		// 		geo_updated = true;
		// 	}

			wc_view_damage_whole(view);

			if (pending_serial == view->surface_type.xdg.xdg_surface->configure_serial) {
				view->pending_serial = 0;
				view->is_pending_serial = false;
			}
		}
		break;
	case Xwayland:
		if (pending_serial > 0) {
			// wc_view_damage_whole(view);

			// if (view->pending_geometry.x != view->geo.x) {
			// 	view->geo.x = view->pending_geometry.x;
			// 	geo_updated = true;
			// }
			// if (view->pending_geometry.y != view->geo.y) {
			// 	view->geo.y = view->pending_geometry.y;
			// 	geo_updated = true;
			// }

			wc_view_damage_whole(view);
			view->pending_serial = 0;
		}
		break;
	}

	if (geo_updated) {
		updated_window(view->server.server->wm, view);
	}
	pixman_region32_fini(&damage);
}

struct View *wc_view_at(struct wc_server *server, double lx, double ly,
		double *out_sx, double *out_sy, struct wlr_surface **out_surface) {
	assert(out_surface != NULL);

	struct View *view;
	wl_list_for_each(view, &server->views, link) {
		if (wc_is_view_at(view, lx, ly, out_sx, out_sy, out_surface)) {
			return view;
		}
	}
	return NULL;
}

void wc_focus_view(struct View *view) {
	assert(view != NULL);

	struct wc_server *server = view->server.server;
	struct wlr_surface *surface = wc_view_surface(view);
	struct wlr_seat *seat = server->seat->seat;
	struct wlr_surface *prev_surface = seat->keyboard_state.focused_surface;
	if (prev_surface == surface) {
		return;
	}

	if (prev_surface && wlr_surface_is_xdg_surface(prev_surface)) {
		struct wlr_xdg_surface *previous =
				wlr_xdg_surface_from_wlr_surface(prev_surface);
		wlr_xdg_toplevel_set_activated(previous, false);
	}

	/* Move the view to the front */
	wl_list_remove(&view->link);
	wl_list_insert(&server->views, &view->link);

	switch (view->surface_type.tag) {
	case Xdg:
		wlr_xdg_toplevel_set_activated(view->surface_type.xdg.xdg_surface, true);
		break;
	case Xwayland:
		wlr_xwayland_surface_activate(view->surface_type.xwayland.xwayland_surface, true);
		break;
	}

	struct wlr_keyboard *keyboard = wlr_seat_get_keyboard(seat);
	if (keyboard != NULL) {
		wlr_seat_keyboard_notify_enter(seat, surface, keyboard->keycodes,
				keyboard->num_keycodes, &keyboard->modifiers);
	}

	advise_focus_gained(server->wm, view);
}

void wc_views_init(struct wc_server *server) {
	wl_list_init(&server->views);
	wc_xdg_init(server);

	// register_focus_callback(server->wm, wc_focus_window);
	// register_dirty_window_callback(server->wm, wc_view_update_geometry_from_wm);
}

// void wc_views_fini(struct wc_server *server) {
// 	struct View *view;
// 	struct View *temp;
// 	wl_list_for_each_safe(view, temp, &server->views, link) {
// 		switch (view->surface_type.tag) {
// 		case Xdg:
// 			wc_xdg_surface_destroy(&view->destroy, NULL);
// 			break;
// 		case Xwayland:
// 			wc_xwayland_surface_destroy(&view->destroy, NULL);
// 			break;
// 		}
// 	}

// 	wc_xdg_fini(server);
// }

void wc_view_for_each_surface(struct View *view,
		wlr_surface_iterator_func_t iterator, void *data) {
	switch (view->surface_type.tag) {
	case Xdg:
		wlr_xdg_surface_for_each_surface(view->surface_type.xdg.xdg_surface, iterator, data);
		break;
	case Xwayland: {
		struct wlr_xwayland_surface *xwayland_surface = view->surface_type.xwayland.xwayland_surface;
		iterator(xwayland_surface->surface, 0, 0, data);
		break;
	}
	}
}
