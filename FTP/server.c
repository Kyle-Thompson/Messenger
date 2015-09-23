#include "header.h"

// TODO use errno in error messages.
// Example:
// 		if(send(client_socket, sendBuff, sizeof(sendBuff), 0) < 0)
//     		printf("Error: Send\nErrno: %d\n", errno);
// TODO look into TCP_CORK
// TODO should sendfile64() be used for actual file transfers?

int main() {
	int server_socket = 0, client_socket = 0;

	struct sockaddr_in serv_addr;

	char recvBuff[BUFFER+1]; // +1 for the null character.
	char command[6];

	char request[6][6] = {"ls", "get", "put", "cd", "mkdir", "err"};
	void (*request_handler[6]) (char* command, int client_socket) = 
												{ ls, get, put, cd, mkdr, err };

	// Connect server_socket to a socket.
	if ((server_socket = socket(AF_INET, SOCK_STREAM, 0)) < 0) {
		printf("socket error\n");
		return 1;
	}

	memset(&serv_addr, '0', sizeof(serv_addr));
	memset(recvBuff, '0', sizeof(recvBuff));

	serv_addr.sin_family = AF_INET;    
	serv_addr.sin_addr.s_addr = htonl(INADDR_ANY); 
	serv_addr.sin_port = htons(PORT);

	if (bind(server_socket, (struct sockaddr*)&serv_addr, sizeof(serv_addr)) < 0) {
		printf("bind error\n");
		return 1;
	}

	if (listen(server_socket, MAXPENDING) < 0) {
		printf("listen error\n");
		return 1;
	}

	if ((client_socket = accept(server_socket, (struct sockaddr*)NULL ,NULL)) < 0) {
		printf("accept error\n");
		return 1;
	}
  

	while (1) {
		if(recv(client_socket, recvBuff, BUFFER, 0) < 0)
			printf("Error: Receive\nErrno: %d\n", errno);
		recvBuff[BUFFER] = '\n';

		sscanf(recvBuff, "%s", command);
		command[5] = '\n';

		if (strcmp(command, "exit") == 0) break;

		// Handle request
		for (int i = 0; i < 6; ++i)
			if (strcmp(command, request[i]) == 0 || i == 5)
				request_handler[i](recvBuff, client_socket);
	}

	close(client_socket);
	close(server_socket); 

	return 0;    
}

// Displays contents of current directory to client
// Stretch goal: allow client to ls directories other than current
// 		e.g. ls test/so_much_bacon/
void ls  (char* command, int client_socket) {

	// call ls and send the result to "ls.txt" and open it
	system("ls > ls.txt");
	int filehandle = open("ls.txt", O_RDONLY);

	// get the size of "ls.txt" (probably a better way to do this)
	struct stat file_info;
	stat("ls.txt", &file_info);
	int size = file_info.st_size;

	// Send the client the size of the file followed by the file itself
	if (send(client_socket, &size, sizeof(int), 0) < 0) {
		printf("send error\n");
		exit(1);
	}
	if (sendfile(client_socket, filehandle, NULL, size) < 0) {
		printf("sendfile error\n");
		exit(1);
	}

	// Clean up ls.txt
	close(filehandle);
	system("rm ls.txt");
}


void get (char* command, int client_socket) {

}

void put (char* command, int client_socket) {

}


void cd  (char* command, int client_socket) {

}


void mkdr(char* command, int client_socket) {

}

// Tell the client that their command was not valid.
void err (char* command, int client_socket) {
// create a file "err.txt" and store the error message in it.
	FILE *fp = fopen("err.txt", "r");

	char cmd[BUFFER+1];
	sscanf(command, "%s", cmd);
	cmd[BUFFER] = '\n';

	fprintf(fp, "Command \"%s\" not recognized\n", cmd);

// get the size of "err.txt" (probably a better way to do this)
	struct stat file_info;
	stat("err.txt", &file_info);
	int size = file_info.st_size;

// Send the client the size of the file followed by the file itself
	if (send(client_socket, &size, sizeof(int), 0) < 0) {
		printf("send error\n");
		exit(1);
	}
	if (sendfile(client_socket, fileno(fp), NULL, size) < 0) {
		printf("sendfile error\n");
		exit(1);
	}

// clean up err.txt
	fclose(fp);
	system("rm err.txt");
}