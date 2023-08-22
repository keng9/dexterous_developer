# Use with the CLI

The CLI allows you to avoid needing to compile your dependencies twice, or needing to set up a separate launcher package for hot reloading your  application. It provides a very simple interface with just a couple of options:

If you are working in a non-workspace package, you can just run `dexterous_developer_cli`.
If you are working in a workspace with multiple libraries set up, you will need to specify the package containing your game with `dexterous_developer_cli -p PACKAGE_NAME`.
If you want to enable or disable features, use `--features` to add the ones you want. Note that "bevy/dynamic_linking" and "dexterous_developer/hot_internal" will always be added, since they are required for the reloading capacity to work.