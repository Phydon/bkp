# bkp

**run backups**

* creates a *bkp folder* in the config directory (if not already exists)
* creates a *bkp.txt* file in the *bkp folder* (if not already exists)
    * place all files and directories that should be included in the backup in this file
    * Usage for the bkp.txt file:
        > <folder_name> = <path_to_source> & <overwrite_bool>
    * Example: 
        > my_backup = C:\\\\Users\\\\Username\\\\Documents\\\\important_folder\\\\ & true
    * <path_to_source> must currently be in the format as provided in the example
    * skips already existing files and folders if nothing has been modified since the last backup
    * if <overwrite_bool> is set to false: 
        * it creates a new folder with the specified <folder_name> and the current datetime as the name of the created folder
    * if <overwrite_bool> is set to true: 
        * it replaces the old files with the new ones
    * if "=" is missing or "&" is missing:
        * returns an error
    * everything placed after "#" or "//" will be treated as a comment and will be ignored
    * every empty line will be ignored
* every information, warnings or errors can be found in the *bkp.log* file in the *bkp folder*

## Installation

### Windows

via Cargo or get the ![binary](https://github.com/Phydon/bkp/releases)

## Keep in mind

**always back up your files and folders before using this program**

*always test if your backup has worked correctly*

***always back up your files and folders***
