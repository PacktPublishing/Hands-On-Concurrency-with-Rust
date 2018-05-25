#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

struct ckms;

struct ckms* alloc_ckms(double error);
void free_ckms(struct ckms* ckms);
void ckms_insert(struct ckms* ckms, float value);
int8_t query(struct ckms* ckms, double q, float* quant);

int main(void) {
  srand(time(NULL));

  struct ckms* ckms = alloc_ckms(0.001);

  for (int i=1; i <= 1000000; i++) {
    ckms_insert(ckms, (float)rand()/(float)(RAND_MAX/10000));
  }

  float quant = 0.0;
  if (query(ckms, 0.75, &quant) < 0) {
    printf("ERROR IN QUERY");
    return 1;
  }
  printf("75th percentile: %f\n", quant);

  free_ckms(ckms);
  return 0;
}
