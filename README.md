
`modenv` is a tool to update and keep consistent multiple .env files. 
It is designed to be extremely user-friendly, with helpful error messages, and importantly, 
no operation is destructive without explicit opt-in (with -f flag.)

# Installation

    cargo install modenv
    
# Quickstart

This is a quick overview of the most important commands. These should cover the vast majority of your day to day usage.

##### Initialize your environment

    modenv init
    
If you already have dotenv files for your project, you can skip this command. This command touches 
.env.example, .env, and .env.production files in the current folder.
 It additionally adds the appropriate lines to your .gitignore file, creating a .gitignore if it does not exist.
    
##### Adding to the environment 

    modenv -a PORT=3000 HOST=0.0.0.0

This adds PORT=5000 and HOST=0.0.0.0 to the first env file found, searching in order only for 
.env.local, .env.development, and .env. 
The -a flag causes this to additionally add the key with a 
blank value to all other found files, keeping them consistent.
If the key already exists, this operation will fail unless -f is also passed.
    
##### Add env variables to production

Next, add values to .env.production (specified by -p):

    modenv -p HOST=0.0.0.0 PORT=5000

##### Verifying consistency

`modenv` also has two commands to ensure your .env files are all consistent with each other. These previous commands
assume the `add` subcommand, which is the default if no subcommand is provided.

    modenv audit
    
This shows us any env variables that are missing. You can specify a reference file using -p for .env.production, 
-n for the bare .env file, -d for .env.development, -x for .env.example, or -e <FILE> to specify a specific filepath.

If we want to update those missing keys with blank values, just re-run with -f:

    modenv audit -f

##### Keep files in sync

It's helpful to keep every file in the same order. Re-arrange .env.example how you like, and use the following:

    modenv order 

Don't worry, if you've already re-arranged .env, you can use it as the reference file.

    modenv order -n .env.example
    
This operation warns if re-ordering would delete any data, and it requires an -f flag to overwrite if any data would be lost.