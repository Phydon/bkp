# bkp

**run automated backups**

* creates a *bkp folder* in the config directory (if not already exists)
* creates a *bkp.txt* file in the *bkp folder* (if not already exists)
* place all files and directories that should be included in the backup in this file
* Usage for the bkp.txt file:
    > <folder_name> = <path_to_source> & <overwrite>
* Example: 
    > my_backup = C:\\Users\\Username\\Documents\\important_folder\\ & true
* <path_to_source> must currently be in the format as provided in the example
* skips already existing files and folders if nothing has been modified since last backup
* if <overwrite> is set to false: 
    * it creates a new folder with the specified <folder_name> and the current datetime as the name of the created folder
* if <overwrite> is set to true: 
    * it replaces the old files with the new ones
* if "=" is missing or "&" is missing:
    * returns an error
* everything placed after "#" or "//" will be threated as a comment and will be ignored
* every information, warning or errors can be found in the *bkp.log* file in the *bkp folder*