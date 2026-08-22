package com.opexence.dto;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;
import java.time.Instant;
import java.util.UUID;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record ShopDto(
        UUID id,
        String slug,
        String name,
        String shopType,
        boolean isSystem,
        Instant createdAt
) {}
