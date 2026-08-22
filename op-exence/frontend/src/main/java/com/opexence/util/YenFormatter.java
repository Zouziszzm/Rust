package com.opexence.util;

import java.text.NumberFormat;
import java.util.Locale;

public final class YenFormatter {

    private static final NumberFormat FORMAT = NumberFormat.getNumberInstance(Locale.JAPAN);

    private YenFormatter() {}

    public static String format(long amount) {
        return "¥" + FORMAT.format(amount);
    }
}
