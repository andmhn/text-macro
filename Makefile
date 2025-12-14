cflags = $$(pkg-config --cflags libadwaita-1) -std=gnu11
libs = $$(pkg-config --libs libadwaita-1)

build_dir = build
main = $(build_dir)/main
objs = $(build_dir)/main.o $(build_dir)/macro.o

all: build $(main)

build:
	mkdir -p $(build_dir)

$(main): $(objs)
	gcc -o $(main) $(objs) $(libs)

#-fsanitize=leak #-fsanitize=address -fsanitize=undefined

$(build_dir)/%.o: %.c
	gcc $(cflags) -c $< -o $@

clean:
	rm $(main) $(objs)
