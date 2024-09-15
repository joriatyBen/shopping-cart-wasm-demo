package main

import "flag"

type Config struct {
	Host            string
	User            string
	Password        string
	Database        string
	Listen          string
	ConnectionExtra string
}

func ParseConfig() Config {
	config := Config{}

	flag.StringVar(&config.Host, "h", "127.0.0.1", "Postgres host")
	flag.StringVar(&config.User, "u", "myuser", "Postgres user")
	flag.StringVar(&config.Password, "p", "p05tgr3$", "Postgres password")
	flag.StringVar(&config.Database, "d", "products", "Postgres database")
	flag.StringVar(&config.Listen, "l", "localhost:3005", "HTTP listen address")
	flag.StringVar(&config.ConnectionExtra, "e", "", "Extra postgres connection parameters")
	flag.Parse()

	return config
}
