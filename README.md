# Redusa

Redusa is a simple high score backend API for games. It is written in Rust, a language developed by Mozilla. 

# Description

The API registers scores for different users organized into levels. The size of the high score list can be configured. Levels are created dynamically. A default authentication interface is provided with a null implementation. The API returns JSON formatted data, and is RESTful. Request bodies are required to be encoded JSON entities. The default backend persists data in memory, but alternate implementations can be provided.

# Dependecies

* rust-http (This will be superceded by Teepee or Iron)

# General Design 

The functions are described in detail below and the notation <value> means a call parameter value or a return value. All calls shall result in the HTTP status code 200, unless when something goes wrong, where the appropriate HTTP code must be returned. Numbers parameters and return values are sent in decimal ASCII representation as expected.

# API 

TODO
