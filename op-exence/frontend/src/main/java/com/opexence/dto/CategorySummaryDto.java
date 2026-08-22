package com.opexence.dto;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;
import java.util.UUID;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record CategorySummaryDto(
        UUID categoryId,
        String categoryName,
        String group,
        long total,
        long count
) {}
