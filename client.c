#include "common.h"
 
int main(void) {
  int sockfd = 0;
  char recvBuff[BUFFER+1];
  char sendBuff[BUFFER+1];
  struct sockaddr_in serv_addr;
 
  memset(recvBuff, '0' ,sizeof(recvBuff));
  if((sockfd = socket(AF_INET, SOCK_STREAM, 0)) < 0) {
    printf("\n Error : Could not create socket \n");
    return 1;
  }
 
  serv_addr.sin_family = AF_INET;
  serv_addr.sin_port = htons(PORT);
  serv_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
 
  if(connect(sockfd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0) {
    printf("\n Error : Connect Failed \n");
    return 1;
  }
 
  pid_t pid = fork();
  if (pid) {
    while (strncmp(recvBuff, "exit", 4) != 0) {
      if(recv(sockfd, recvBuff, sizeof(recvBuff), 0) < 0)
        printf("Error: Receive\nErrno: %d\n", errno);
      recvBuff[BUFFER] = '\n';

      printf("Server: %s", recvBuff);
    }
    
    wait(&pid);
    close(sockfd);
    
  } else {
    while (strncmp(sendBuff, "exit", 4) != 0) {
      fputs("(Client) Enter a message: ", stdout);
      fgets(sendBuff, sizeof(sendBuff), stdin);
      
      if(send(sockfd, sendBuff, sizeof(sendBuff), 0) < 0)
        printf("Error: Send\nErrno: %d\n", errno);
    }
    close(sockfd);
    exit(0);
  }
  
  return 0;
}