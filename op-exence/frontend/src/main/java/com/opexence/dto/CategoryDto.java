package com.opexence.dto;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;
import java.time.Instant;
import java.util.UUID;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public record CategoryDto(
        UUID id,
        String slug,
        String name,
        String categoryGroup,
        String description,
        boolean isSystem,
        Instant createdAt,
        Instant updatedAt
) {}
