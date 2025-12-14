#include <glib.h>

typedef struct {
    GString *inner_text;
} Text;

Text * text_init();
void text_deinit(Text * t);
void text_set(Text* t, const char* str);
void text_repeat(Text* t, size_t times);
void text_reset(Text* t);
