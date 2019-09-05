#include "xdg.h"

#include <stdint.h>
#include <stdlib.h>

#include <wayland-server.h>
#include <wlr/types/wlr_xdg_shell.h>
#include <wlr/util/log.h>

#include "../rust_wm/rust_wm.h"

#include "cursor.h"
#include "output.h"
#include "seat.h"
#include "server.h"
#include "view.h"

static void wc_xdg_surface_map(struct wl_listener *listener, void *data) {
	struct View *view = wl_container_of(listener, view, map);
	view->mapped = true;
	// wc_focus_view(view);

	struct wlr_xdg_surface *surface = view->surface_type.xdg.xdg_surface;
	struct wlr_box box = {0};
	wlr_xdg_surface_get_geometry(surface, &box);
	memcpy(&view->geo, &box, sizeof(struct wlr_box));
	handle_window_ready(view->server.server->wm, view);

	struct wlr_xdg_toplevel *toplevel = view->surface_type.xdg.xdg_surface->toplevel;
	printf("2current fullscreen: %d, maximized: %d\n", toplevel->current.fullscreen, toplevel->current.maximized);
	printf("2server_pending fullscreen: %d, maximized: %d\n", toplevel->server_pending.fullscreen, toplevel->server_pending.maximized);
	printf("2client_pending fullscreen: %d, maximized: %d\n", toplevel->client_pending.fullscreen, toplevel->client_pending.maximized);

	// bool is_tiled = configure_window(view->server.server->wm, view->window_id, &view->geo, toplevel->app_id, toplevel->client_pending.fullscreen);
	if (view->geo.width != view->surface_type.xdg.xdg_surface->geometry.width || view->geo.height != view->surface_type.xdg.xdg_surface->geometry.height) {
		wlr_xdg_toplevel_set_size(view->surface_type.xdg.xdg_surface, view->geo.width, view->geo.height);
	}
	// if (is_tiled) {
	// 	wlr_xdg_toplevel_set_tiled(
	// 		view->surface_type.xdg.xdg_surface, 
	// 		WLR_EDGE_TOP & 
	// 		WLR_EDGE_BOTTOM & 
	// 		WLR_EDGE_LEFT & 
	// 		WLR_EDGE_RIGHT
	// 	);
	// }

	wc_view_commit(view, view->geo);
}

static void wc_xdg_surface_unmap(struct wl_listener *listener, void *data) {
	struct View *view = wl_container_of(listener, view, unmap);
	view->mapped = false;

	wc_view_damage_whole(view);
}

static void wc_xdg_surface_commit(struct wl_listener *listener, void *data) {
	struct View *view = wl_container_of(listener, view, commit);
	struct wlr_xdg_surface *xdg_surface = view->surface_type.xdg.xdg_surface;

	struct wlr_box geo = {0};
	wlr_xdg_surface_get_geometry(xdg_surface, &geo);

	wc_view_commit(view, geo);
}

void wc_xdg_surface_destroy(struct wl_listener *listener, void *data) {
	struct View *view = wl_container_of(listener, view, destroy);

	advise_delete_window(view->server.server->wm, view);

	wl_list_remove(&view->link);

	wl_list_remove(&view->map.link);
	wl_list_remove(&view->unmap.link);
	wl_list_remove(&view->commit.link);
	wl_list_remove(&view->request_move.link);
	wl_list_remove(&view->request_resize.link);
	wl_list_remove(&view->destroy.link);

	free(view);
}

static void wc_xdg_toplevel_request_move(
		struct wl_listener *listener, void *data) {
	struct View *view = wl_container_of(listener, view, request_move);

	struct wlr_box geo;
	wlr_xdg_surface_get_geometry(view->surface_type.xdg.xdg_surface, &geo);

	wc_view_move(view, geo);
}

static void wc_xdg_toplevel_request_resize(
		struct wl_listener *listener, void *data) {
	struct View *view = wl_container_of(listener, view, request_resize);
	struct wlr_xdg_toplevel_resize_event *event = data;

	struct wlr_box geo;
	wlr_xdg_surface_get_geometry(view->surface_type.xdg.xdg_surface, &geo);

	wc_view_resize(view, geo, event->edges);
}

static void wc_xdg_new_surface(struct wl_listener *listener, void *data) {
	struct wc_server *server =
			wl_container_of(listener, server, new_xdg_surface);
	struct wlr_xdg_surface *xdg_surface = data;
	if (xdg_surface->role != WLR_XDG_SURFACE_ROLE_TOPLEVEL) {
		return;
	}

	View* view = create_xdg_window(server->wm, xdg_surface);
	view->server.server = server;

	view->map.notify = wc_xdg_surface_map;
	view->unmap.notify = wc_xdg_surface_unmap;
	view->commit.notify = wc_xdg_surface_commit;
	view->destroy.notify = wc_xdg_surface_destroy;

	wl_signal_add(&xdg_surface->events.map, &view->map);
	wl_signal_add(&xdg_surface->events.unmap, &view->unmap);
	wl_signal_add(&xdg_surface->surface->events.commit, &view->commit);
	wl_signal_add(&xdg_surface->events.destroy, &view->destroy);

	struct wlr_xdg_toplevel *toplevel = xdg_surface->toplevel;
	view->request_move.notify = wc_xdg_toplevel_request_move;
	wl_signal_add(&toplevel->events.request_move, &view->request_move);
	view->request_resize.notify = wc_xdg_toplevel_request_resize;
	wl_signal_add(&toplevel->events.request_resize, &view->request_resize);

	wl_list_insert(&server->views, &view->link);

	printf("current fullscreen: %d, maximized: %d\n", toplevel->current.fullscreen, toplevel->current.maximized);
	printf("server_pending fullscreen: %d, maximized: %d\n", toplevel->server_pending.fullscreen, toplevel->server_pending.maximized);
	printf("client_pending fullscreen: %d, maximized: %d\n", toplevel->client_pending.fullscreen, toplevel->client_pending.maximized);
}

void wc_xdg_init(struct wc_server *server) {
	server->xdg_shell = wlr_xdg_shell_create(server->wl_display);
	server->new_xdg_surface.notify = wc_xdg_new_surface;
	wl_signal_add(
			&server->xdg_shell->events.new_surface, &server->new_xdg_surface);
}

void wc_xdg_fini(struct wc_server *server) {
	wlr_xdg_shell_destroy(server->xdg_shell);
	server->xdg_shell = NULL;

	wl_list_remove(&server->new_xdg_surface.link);
}
