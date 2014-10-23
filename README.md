# Redusa [![Build Status](https://travis-ci.org/elyzion/redusa.svg?branch=master)](https://travis-ci.org/elyzion/redusa)

Redusa is a simple high score backend API for games. It is written in Rust, a language developed by Mozilla. 

# Description

The API registers scores for different users organized into levels. The size of the high score list can be configured. Levels are created dynamically. The API returns JSON formatted data. Request bodies are required to be encoded JSON entities. The default backend persists data in memory.

# Dependecies

* Iron

# General Design 

The functions are described in detail below. All calls shall result in the HTTP status code 200, unless when something goes wrong, where the appropriate HTTP code must be returned. Numerical parameters and return values are sent in decimal ASCII representation.

# API 

## Get a list of levels

Signature: GET /level

## Get the user and point list for a particular level

Signature: GET /level/:level

## Add an entry to a level

Signature: POST /level/:level
Body: Json object for user and points

TODO

* Authentication trait with a default implementation.
* Extraction of backend trait so that we can have different kinds of persistence models.
* Tests for the iron part of the program.
