cargo = find_program('cargo', required : true)

if get_option('release')
    message('Release mode enabled')
    cargo_args = ['build', '--release']
    grab_objects_arg = 'release'
else
    message('Debug mode enabled')
    cargo_args = ['build']
    grab_objects_arg = 'debug'
endif

message('Running cargo...')
cargo_result = run_command(cargo, cargo_args)
cargo_stdout = cargo_result.stdout().strip()
cargo_stderr = cargo_result.stderr().strip()
if cargo_stderr != ''
    message(cargo_stderr)
elif cargo_stdout != ''
    message(cargo_stdout)
else
    message('Cargo is done')
endif


# Here we grab any shared library objects from Cargo's generated "target" directory
# using a shell script to take advantage of wildcards.
grab_objects = find_program('grab_objects.sh')

# The output is sanitized and made into an array.
cargo_rslib_objects = run_command(grab_objects, grab_objects_arg).stdout().strip().split('\n')

message('Found objects:')
message(cargo_rslib_objects)