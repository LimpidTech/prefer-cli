prefer-cli
==========

A tool for exploring prefer configuration data.


Installation
------------

Installation is simple.

    npm install prefer-cli


Usage
-----

This tool allows you to load a configuration through [prefer][prfr]. It will
displays each level of the configuration as a list, and you can drill down into
specific configuration items to see more details about them. This can be
helpful when writing software - or using software - that loads configuration
data, because it allows you to see the values and types of all items in the
configuration tree without needing to manually debug the code.

You can simply run it by passing any configuration file to the prefer
executable:

    prefer myfile.yml

Any configuration file recognized by [prefer][prfr] can be used here, so any of
the following are possible use cases (and many more):

    prefer myfile.ini
    prefer myfile.json
    prefer sqlite://filename.sqlite3


### Default key bindings

- j: navigate down
- k: navigate up
- h: back
- q or escape: quit


[prfr]: http://github.com/LimpidTech/prefer
