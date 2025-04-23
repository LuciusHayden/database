# Database Implementation

Write Ahead Log

Collections 

Authentication / Sessions 

Custom errors 

Command Line Interface 


# currently supported operations (commands are non case sensitive)
INSERT (key) (value)

GET (key)

DELETE (key)

SELECT (collection)

NEW (collection)

WHICH (collection/path/user)


# CLI arguments
-u (username)

    username for the user being used 

-p (password) 
    
    password for the user being used 

-d (directory) default="./data"
    
    specifies the directory for the database to be formed from

--new-user default=false
    
    takes the username and password and attempts to make a new user 


