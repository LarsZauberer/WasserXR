#include "TheSeed/components/Mesh.h"
#include "TheSeed/ecs/Scene.h"
#include <stdlib.h>

void *ts_create_TS_Mesh() {
  TS_Mesh *mesh = (TS_Mesh *)malloc(sizeof(TS_Mesh));

  return mesh;
}

void ts_destroy_TS_Mesh(void *mesh) {
  free(mesh);
  return;
}
