#include "TheSeed/components/Mesh.h"
#include "TheSeed/components/Transform.h"
#include "TheSeed/core/Shader.h"
#include "TheSeed/ecs/Scene.h"
#include "glad/gl.h"
#include <stdio.h>

int ts_select_ts_mesh_renderer(TS_Scene_t *scene, const size_t entity) {
  if (ts_entity_get_component(scene, entity, "TS_Transform") &&
      ts_entity_get_component(scene, entity, "TS_Mesh")) {
    return 1;
  } else {
    return 0;
  }
}

void ts_system_ts_mesh_renderer(TS_Scene_t *scene, size_t *entities, size_t n) {
  for (size_t i = 0; i < n; i++) {
    size_t entity = entities[i];
    TS_Mesh *mesh = ts_entity_get_component(scene, entity, "TS_Mesh");
    TS_Transform_t *transform =
        ts_entity_get_component(scene, entity, "TS_Transform");

    ts_use_shader(mesh->shader);
    glBindVertexArray(mesh->vao);
    glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_INT, 0);
  }
  return;
}
