// Copyright (c) 2025 Dave Parfitt

any {
    test true! {}
    test false! {}
}


all {
    all {
        test true! {}
        test true! {}
    }

    any {
        test true! {}
        test false! {}
    }

    none {
        test false! {}
        test false! {}
    }
}
