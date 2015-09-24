C = gcc -Wall -std=c11 -o

all:	client server

client:	client.c
	$(C) client client.c

server:	server.c
	$(C) server server.c

clean:	
	rm -f client server
