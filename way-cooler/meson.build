cc = meson.get_compiler('c')

wayland_server = dependency('wayland-server')
wayland_client = dependency('wayland-client')
wayland_cursor = dependency('wayland-cursor')
wayland_egl    = dependency('wayland-egl')
wayland_protos = dependency('wayland-protocols', version: '>=1.14')
xkbcommon      = dependency('xkbcommon')
xcb            = dependency('xcb')
pixman         = dependency('pixman-1')
libinput       = dependency('libinput', version: '>=1.6.0')

# Try first to find wlroots as a subproject, then as a system dependency
wlroots_version = '>=0.5.0'
wlroots_proj = subproject(
	'wlroots',
	default_options: ['rootston=false', 'examples=false'],
	required: false,
	version: wlroots_version,
)
if wlroots_proj.found()
	wlroots = wlroots_proj.get_variable('wlroots')
	wlroots_conf = wlroots_proj.get_variable('conf_data')
	wlroots_has_xwayland = wlroots_conf.get('WLR_HAS_XWAYLAND') == 1
else
	wlroots = dependency('wlroots', version: wlroots_version)
	wlroots_has_xwayland = cc.get_define('WLR_HAS_XWAYLAND', prefix: '#include <wlr/config.h>', dependencies: wlroots) == '1'
endif

if not wlroots_has_xwayland
	error('Cannot build compositor: wlroots has been built without Xwayland support')
endif

way_cooler_deps = [
	wayland_server,
	wlroots,
	xkbcommon,
	server_protos,
	pixman,
	libinput,
]

way_cooler_sources = files(
	'cursor.c',
	'input.c',
	'keybindings.c',
	'keyboard.c',
	'layer_shell.c',
	'main.c',
	'mousegrabber.c',
	'output.c',
	'pointer.c',
	'seat.c',
	'server.c',
	'view.c',
	'xdg.c',
	'xkb_hash_set.c',
	'xwayland.c',
)

executable(
    'way-cooler',
    way_cooler_sources,
    install : true,
    include_directories: [include_directories('.')],
	dependencies: way_cooler_deps,
	objects : cargo_rslib_objects,
)
