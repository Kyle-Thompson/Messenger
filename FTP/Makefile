C = gcc -Wall -std=c11 -D_POSIX_SOURCE -o

all:	client server

client:	client.c header.h
	$(C) client client.c 

server:	server.c header.h
	$(C) server server.c 

clean:	
	rm -f client server
