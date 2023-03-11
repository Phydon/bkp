# bkp

**run backups**

* creates a *bkp folder* in the config directory (if not already exists)
* creates a *bkp.txt* file in the *bkp folder* (if not already exists)
    * place all files and directories that should be included in the backup in this file
    * skips already existing files and folders if nothing has been modified since the last backup
    
    * Usage for the config file bkp.txt:
        ```<folder_name> = <path_to_source>, <path_to_destination>, \<overwrite>,```
        
        ```If <path_to_destination> is "default", the backup will be stored in the config folder,```
    * Example:,
        ```my_default_backup = C:/Users/Username/path_to_source/important_folder/, default, true,```
        
        ```my_google_drive_backup = C:/Users/Username/path_to_source/important_folder/, G:/Google_Drive/My Storage/path_to_destination/, true```
        
    * if ```<overwrite>``` is set to *false*: 
        * it creates a new folder with the specified ```<folder_name>``` and the current datetime as the name of the created folder
    * if ```<overwrite>``` is set to *true*: 
        * it replaces the old files with the new ones
    * if ```"="``` is missing:
        * returns an error
    * everything placed after ```"#"``` or ```"//"``` will be treated as a comment and will be ignored
    * every empty line will be ignored
* every information, warnings or errors can be found in the *bkp.log* file in the *bkp folder*

## Usecase

1. local backups 
2. backups via Google Drive synchronization on your local machine: 
    * Google Drive is unable to detect changes in *encrypted files*
    * that means, that encrypted files only get uploaded once and will **not** be synchronized if you modify the encrpted file
    * however, with Google Drive Desktop you can create a local drive and everything in there will be uploaded to your cloud storage
    * set this drive as your destination path in the *bkp.txt* config file and you can get automated backups via Google Drive

## Installation

### Windows

via Cargo or get the ![binary](https://github.com/Phydon/bkp/releases)

## Keep in mind

**always back up your files and folders before using this program**

*always test if your backup has worked correctly*

***always back up your files and folders***
