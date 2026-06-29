__thread int libvar = 99;

int get_libvar(void) {
    return libvar;
}

void set_libvar(int v) {
    libvar = v;
}
