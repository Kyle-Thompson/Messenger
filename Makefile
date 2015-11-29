C = g++ -Wall -std=c++14 -o

all:	client server

client:	client.cpp common.h
	$(C) client client.cpp wrappers.cpp

server:	server.cpp common.h
	$(C) server server.cpp wrappers.cpp

clean:	
	rm -f client server
