package com.opexence.controller;

import com.opexence.client.ApiClient;
import com.opexence.client.ApiException;
import com.opexence.dto.CreateExpenseRequest;
import com.opexence.dto.ExpenseDto;
import com.opexence.dto.ExpenseSummaryResponse;
import com.opexence.dto.GroupSummaryDto;
import com.opexence.util.YenFormatter;
import java.time.Instant;
import java.time.LocalDate;
import java.time.ZoneId;
import java.time.temporal.TemporalAdjusters;
import java.util.List;
import org.springframework.stereotype.Controller;
import org.springframework.ui.Model;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.servlet.mvc.support.RedirectAttributes;

@Controller
public class DashboardController {

    private final ApiClient apiClient;

    public DashboardController(ApiClient apiClient) {
        this.apiClient = apiClient;
    }

    @GetMapping("/")
    public String dashboard(Model model) {
        ZoneId zone = ZoneId.of("Asia/Tokyo");
        LocalDate now = LocalDate.now(zone);
        Instant from = now.with(TemporalAdjusters.firstDayOfMonth()).atStartOfDay(zone).toInstant();
        Instant to = now.plusDays(1).atStartOfDay(zone).toInstant();

        ExpenseSummaryResponse summary = apiClient.getSummary(from, to, null);
        List<ExpenseDto> recent = apiClient.listExpenses(null, null, null, null, null, null, null, 10, 0).items();

        long housingTotal = sumGroup(summary.byGroup(), "housing")
                + sumGroup(summary.byGroup(), "parking");
        long groceriesTotal = sumGroup(summary.byGroup(), "groceries");
        long deductionsTotal = summary.taxBreakdown().salaryDeductionsTotal();

        model.addAttribute("summary", summary);
        model.addAttribute("recentExpenses", recent);
        model.addAttribute("housingTotal", YenFormatter.format(housingTotal));
        model.addAttribute("groceriesTotal", YenFormatter.format(groceriesTotal));
        model.addAttribute("deductionsTotal", YenFormatter.format(deductionsTotal));
        model.addAttribute("consumptionTaxTotal", YenFormatter.format(summary.taxBreakdown().taxTotal()));
        model.addAttribute("monthTotal", YenFormatter.format(summary.total()));
        model.addAttribute("monthLabel", now.getMonth().toString() + " " + now.getYear());
        return "dashboard";
    }

    private long sumGroup(List<GroupSummaryDto> groups, String name) {
        if (groups == null) {
            return 0;
        }
        return groups.stream()
                .filter(g -> g.group() != null && g.group().equals(name))
                .mapToLong(GroupSummaryDto::total)
                .sum();
    }

    @GetMapping("/error")
    public String error(@RequestParam(required = false) String message, Model model) {
        model.addAttribute("message", message != null ? message : "Something went wrong");
        return "error";
    }

    static void flashApiError(RedirectAttributes redirect, ApiException ex) {
        redirect.addFlashAttribute("errorMessage", ex.getMessage());
    }
}
