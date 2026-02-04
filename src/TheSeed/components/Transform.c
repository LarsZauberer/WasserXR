#include "TheSeed/components/Transform.h"
#include <stdlib.h>

void *ts_create_TS_Transform() {
  TS_Transform_t *t = (TS_Transform_t *)malloc(sizeof(TS_Transform_t));

  t->x = 1.0;
  t->y = 2.0;
  t->z = 3.0;

  return t;
}
