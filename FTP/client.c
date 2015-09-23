#include "header.h"

int main(void)
{
  printf("I AM THE CLIENT\n");
  int sockfd = 0;
  char recvBuff[1024];
  char sendBuff[1024];
  struct sockaddr_in serv_addr;
 
  memset(recvBuff, '0' ,sizeof(recvBuff));
  if((sockfd = socket(AF_INET, SOCK_STREAM, 0)) < 0) {
    printf("\n Error : Could not create socket \n");
    return 1;
  }
 
  serv_addr.sin_family = AF_INET;
  serv_addr.sin_port = htons(5000);
  serv_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
 
  if(connect(sockfd, (struct sockaddr *)&serv_addr, sizeof(serv_addr)) < 0) {
    printf("\n Error : Connect Failed \n");
    return 1;
  }
 
  pid_t pid = fork();
  if (pid) {
    while (strncmp(recvBuff, "exit", 4) != 0) {
      // why do i have to have +1 to the sizeof()?????????
      if(recv(sockfd, recvBuff, sizeof(recvBuff) +1, 0) < 0)
        printf("Error: Receive\nErrno: %d\n", errno);
      recvBuff[1023] = 0;

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
