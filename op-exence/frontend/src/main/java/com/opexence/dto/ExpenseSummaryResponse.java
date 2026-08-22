package com.opexence.dto;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;
import java.util.Collections;
import java.util.List;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record ExpenseSummaryResponse(
        long total,
        List<GroupSummaryDto> byGroup,
        List<CategorySummaryDto> byCategory,
        List<ShopSummaryDto> byShop,
        TaxBreakdownDto taxBreakdown
) {
    public List<GroupSummaryDto> byGroup() {
        return byGroup != null ? byGroup : Collections.emptyList();
    }

    public List<CategorySummaryDto> byCategory() {
        return byCategory != null ? byCategory : Collections.emptyList();
    }

    public List<ShopSummaryDto> byShop() {
        return byShop != null ? byShop : Collections.emptyList();
    }

    public TaxBreakdownDto taxBreakdown() {
        return taxBreakdown != null ? taxBreakdown : new TaxBreakdownDto(0, 0, 0, 0);
    }
}
