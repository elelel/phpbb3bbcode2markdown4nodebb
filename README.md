# phpbb3bbcode2markdown4nodebb
Post-process imported from PhpBB  NodeBB database 

*Features:*
* Convert multilevel BBCode tags like size, quote, etc
* Tries to make be consistent over line breaks
* Attachment image urls are mapped to filenames that can be recovered through original PhpBB database for postprocessing (moving to upload dir, etc)

*Requires:*
* MongoDB-based NodeBB installation

*Instructions:*

1. Use NodeBB import plugin as instructed on their page on NodeBB ver 1.0.0.
It's recommended to use my nodebb-plugin-import-phpbb fork (imports user avatars)
2. Do NOT use phpbb-bbcode-to-markdown (corrupts formatting). Avoid other postprocessing plugins.
3. Check main.rs and nodebb/mod.rs for connection settings to your mongodb
4. Backup your database, e.g. with mongodump
5. cargo run

