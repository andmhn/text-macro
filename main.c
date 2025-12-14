#include "gio/gio.h"
#include "macro.h"
#include <adwaita.h>
#include <glib.h>
#include <gtk/gtk.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

static Text *text;
static GtkWindow *window;
static GtkEntryBuffer *file_entry_buffer;
static GtkWidget *status_area;
static GtkTextBuffer *buffer;
static GtkWidget *times_input;

static size_t grab_times_value() { return gtk_spin_button_get_value((GtkSpinButton *)times_input); }

static void append_to_file_path() {
    const char *filepath = gtk_entry_buffer_get_text(file_entry_buffer);
    GString *s = g_string_new("");

    GFile *gfile = g_file_new_for_path(filepath);
    if (!g_file_query_exists(gfile, NULL)) { // g_file_query_file_type
        g_string_printf(s, "file doesn't exist: %s", filepath);
        gtk_label_set_text((GtkLabel *)status_area, s->str);
        return;
    }
    GError *error = NULL;
    GFileOutputStream *stream = g_file_append_to(gfile, G_FILE_CREATE_NONE, NULL, &error);
    if (error) {
        gtk_label_set_text((GtkLabel *)status_area, "Error Occured while Opening Stream!!!");
        return;
    }

    GtkTextIter start, end;
    gtk_text_buffer_get_start_iter(buffer, &start);
    gtk_text_buffer_get_end_iter(buffer, &end);
    const char *str = gtk_text_buffer_get_text(buffer, &start, &end, true);
    const gint len = gtk_text_buffer_get_char_count(buffer);

    gssize bytes = g_output_stream_write((GOutputStream *)stream, str, len, NULL, &error);
    if (error) {
        gtk_label_set_text((GtkLabel *)status_area, "Error Occured while Appending!!!");
        return;
    }

    g_string_printf(s, "appended %ld bytes to file: %s", bytes, filepath);
    gtk_label_set_text((GtkLabel *)status_area, s->str);
    free(s);
    g_free((gpointer)str);
}

static void select_file_path(GtkWidget *widget, GtkEntryBuffer *data) {
    const char *text = gtk_entry_buffer_get_text(data);
    if (gtk_entry_buffer_get_length(data) > 0) {
        GString *s = g_string_new("");
        g_string_printf(s, "file selected: %s", text);
        gtk_label_set_text((GtkLabel *)status_area, s->str);

        // TODO: (Optional) load to buffer
    }
}

static void open_picker_callback(GObject *source_object, GAsyncResult *res, gpointer data) {
    GError *error = NULL;
    GFile *f = gtk_file_dialog_open_finish((GtkFileDialog *)source_object, res, &error);
    if (error && error->code == GTK_DIALOG_ERROR_DISMISSED) {
        g_error_free(error);
        return;
    }
    char *path = g_file_get_path(f);
    gtk_entry_buffer_set_text(data, path, strlen(path));
    select_file_path(NULL, data);
}

static void open_picker(GtkWidget *widget, GtkEntryBuffer *data) {
    GtkFileDialog *fd = gtk_file_dialog_new();
    gtk_file_dialog_open(fd, window, NULL, open_picker_callback, data);
}

static void create_top_bar(GtkWidget *box) {
    GtkWidget *file_entry = gtk_entry_new();
    gtk_entry_set_placeholder_text(GTK_ENTRY(file_entry), "Enter the file path here...");
    gtk_widget_set_hexpand(file_entry, TRUE);

    file_entry_buffer = gtk_entry_get_buffer((GtkEntry *)file_entry);
    g_signal_connect(file_entry, "activate", G_CALLBACK(select_file_path), file_entry_buffer);

    GtkWidget *picker_btn;
    picker_btn = gtk_button_new_with_label("Select File");
    gtk_widget_set_halign(picker_btn, GTK_ALIGN_END);
    g_signal_connect(picker_btn, "clicked", G_CALLBACK(open_picker), file_entry_buffer);

    gtk_box_append((GtkBox *)box, file_entry);
    gtk_box_append((GtkBox *)box, picker_btn);
}

static void repeat(GtkWidget *widget, gpointer data) {
    GtkTextIter start, end;
    gtk_text_buffer_get_start_iter(buffer, &start);
    gtk_text_buffer_get_end_iter(buffer, &end);
    const char *str = gtk_text_buffer_get_text(buffer, &start, &end, true);
    if (gtk_text_buffer_get_char_count(buffer) == 0) {
        gtk_label_set_text((GtkLabel *)status_area, "skipping! Reason: empty text area");
        return;
    }
    size_t times = grab_times_value();
    text_set(text, str);
    text_repeat(text, times);
    gtk_text_buffer_set_text(buffer, text->inner_text->str, text->inner_text->len);

    GString *s = g_string_new("");
    g_string_printf(s, "repeated %zu times", times);
    gtk_label_set_text((GtkLabel *)status_area, s->str);
}

static void create_action_bar(GtkWidget *box2) {
    GtkWidget *n_label = gtk_label_new("times ");
    GtkAdjustment *adjustment = gtk_adjustment_new(2, 0, 999, 1, 10, 0);
    times_input = gtk_spin_button_new(adjustment, 1, 0);
    gtk_widget_set_halign(times_input, GTK_ALIGN_START);

    GtkWidget *repeat_btn;
    repeat_btn = gtk_button_new_with_label("Repeat Text");
    g_signal_connect(repeat_btn, "clicked", G_CALLBACK(repeat), 0);

    GtkWidget *spacer = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_set_hexpand(spacer, true);

    GtkWidget *append_btn;
    append_btn = gtk_button_new_with_label("Append Text to File");
    gtk_widget_set_halign(append_btn, GTK_ALIGN_END);
    g_signal_connect(append_btn, "clicked", G_CALLBACK(append_to_file_path), 0);

    gtk_box_append((GtkBox *)box2, n_label);
    gtk_box_append((GtkBox *)box2, times_input);
    gtk_box_append((GtkBox *)box2, repeat_btn);
    gtk_box_append((GtkBox *)box2, spacer);
    gtk_box_append((GtkBox *)box2, append_btn);
}

static void create_editor_ui(GtkWindow *window) {
    GtkWidget *grid = gtk_grid_new();
    gtk_grid_set_row_spacing(GTK_GRID(grid), 10);
    gtk_grid_set_column_spacing(GTK_GRID(grid), 10);
    gtk_widget_set_margin_top(grid, 10);
    gtk_widget_set_margin_bottom(grid, 10);
    gtk_widget_set_margin_start(grid, 10);
    gtk_widget_set_margin_end(grid, 10);
    gtk_window_set_child(GTK_WINDOW(window), grid);

    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
    create_top_bar(box);

    GtkWidget *spacer = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);

    GtkWidget *desc = gtk_label_new("Text to Input/Repeat:");
    gtk_widget_set_halign(desc, GTK_ALIGN_START);

    GtkWidget *text_area = gtk_text_view_new();
    buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(text_area));
    gtk_text_view_set_left_margin((GtkTextView *)text_area, 3);
    GtkWidget *scrolled_text_area = gtk_scrolled_window_new();
    gtk_scrolled_window_set_child((GtkScrolledWindow *)scrolled_text_area, text_area);
    gtk_widget_set_hexpand(scrolled_text_area, TRUE);
    gtk_widget_set_vexpand(scrolled_text_area, TRUE);
    gtk_widget_add_css_class(scrolled_text_area, "scrolled_text_area");

    GtkWidget *box2 = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
    create_action_bar(box2);

    status_area = gtk_label_new("");
    gtk_widget_set_halign(status_area, GTK_ALIGN_START);

    // clang-format off
    gtk_grid_attach(GTK_GRID(grid), box,                0, 0, 1, 1);
    gtk_grid_attach(GTK_GRID(grid), spacer,             0, 1, 1, 1);
    gtk_grid_attach(GTK_GRID(grid), desc,               0, 2, 1, 1);
    gtk_grid_attach(GTK_GRID(grid), scrolled_text_area, 0, 3, 1, 1);
    gtk_grid_attach(GTK_GRID(grid), box2,               0, 4, 1, 1);
    gtk_grid_attach(GTK_GRID(grid), status_area,        0, 5, 1, 1);
    // clang-format on
}

const char *css = ".scrolled_text_area {\n"
                  "    border-style: solid;\n"
                  "    border-width: 2px;\n"
                  "    border-color: rgba(233, 84, 32, 0.4);\n" // ubuntu color
                  "    border-radius: 5px;\n"
                  "    padding: 1px;\n"
                  "}\n";

static void load_css(void) {
    GtkCssProvider *provider = gtk_css_provider_new();
    GdkDisplay *display = gdk_display_get_default();

    gtk_css_provider_load_from_string(provider, css);

    gtk_style_context_add_provider_for_display(display, GTK_STYLE_PROVIDER(provider),
                                               GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);
    g_object_unref(provider);
}

static void activate(GtkApplication *app, gpointer gm) {
    load_css();
    window = (GtkWindow *)gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(window), "Macro");
    gtk_window_set_default_size(GTK_WINDOW(window), 500, 400);

    create_editor_ui(window);
    gtk_window_present(GTK_WINDOW(window));
}

int main(int argc, char **argv) {
    AdwApplication *app;
    int status;

    text = text_init();

    app = adw_application_new("com.github.andmhn.macro", G_APPLICATION_DEFAULT_FLAGS);
    g_signal_connect(app, "activate", G_CALLBACK(activate), text);
    status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);

    text_deinit(text);
    return status;
}
