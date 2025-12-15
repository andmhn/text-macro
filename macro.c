#include "macro.h"
#include "glib.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>

Text * text_init(){
    Text * t = malloc(sizeof(Text));
    t->inner_text = g_string_new("");
    return t;
}

void text_deinit(Text * t){
    g_string_free(t->inner_text, TRUE);
    free(t);
}

void text_set(Text* t, const char* str){
    g_string_free(t->inner_text, TRUE);
    t->inner_text = g_string_new(str);
}

void text_repeat(Text* t, size_t times){
    GString* repeats = g_string_new("");
    for (size_t i = 1; i < times; i++) {
        g_string_append(repeats, t->inner_text->str);
    }
    g_string_append(t->inner_text, repeats->str);
}

void text_reset(Text* t){
    g_string_truncate(t->inner_text, t->inner_text->len);
}
