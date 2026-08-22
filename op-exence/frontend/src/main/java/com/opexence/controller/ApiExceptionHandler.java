package com.opexence.controller;

import com.opexence.client.ApiException;
import org.springframework.web.bind.annotation.ControllerAdvice;
import org.springframework.web.bind.annotation.ExceptionHandler;
import org.springframework.web.servlet.mvc.support.RedirectAttributes;

@ControllerAdvice
public class ApiExceptionHandler {

    @ExceptionHandler(ApiException.class)
    public String handleApiException(ApiException ex, RedirectAttributes redirect) {
        redirect.addFlashAttribute("errorMessage", ex.getMessage());
        return "redirect:/";
    }
}
