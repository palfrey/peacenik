Peacenik
========
[![Build Status](https://travis-ci.org/palfrey/peacenik.svg?branch=master)](https://travis-ci.org/palfrey/peacenik)

A [Beatnik](https://esolangs.org/wiki/Beatnik) interpreter and generator

Usage
-----
`peacenik <command>` (or `cargo run -- <command>` if you're running from source)

Commands
--------
* `run` - Run a Beatnik program
* `wottasquare` - Run a [Wottasquare](https://github.com/catseye/Beatnik#wottasquarepy) program
* `wottasquare-dumper` - Dump a Beatnik program to Wottasquare form
* `generate-markov` - Generate Markov chain information from a source text
* `markov-beatnik` - Given a Markov chain and a Wottasquare program, generate the equivalent Beatnik program